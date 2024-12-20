use crate::error::OrbitlenError;
use crate::events::*;
use crate::state::*;
use anchor_lang::prelude::*;
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;

/// Calculations:
/// `q_lf = q_ll = q_a * p_a * 1 / p_l`
///
/// Expected remaining account schema
/// [
///    asset_oracle_ai,
///    liab_oracle_ai,
///  ]
pub fn lending_account_liquidate_process<'info>(
    ctx: Context<'_, '_, 'info, 'info, LendingAccountLiquidate<'info>>,
    asset_amount: u64
) -> Result<()> {
    require_gt!(asset_amount, 0, OrbitlenError::IllegalLiquidation);

    require_keys_neq!(
        ctx.accounts.asset_bank.key(),
        ctx.accounts.liab_bank.key(),
        OrbitlenError::IllegalLiquidation
    );

    let LendingAccountLiquidate {
        liquidator_orbitlen_account: liquidator_orbitlen_account_loader,
        liquidatee_orbitlen_account: liquidatee_orbitlen_account_loader,
        ..
    } = ctx.accounts;

    let mut liquidator_orbitlen_account = liquidator_orbitlen_account_loader.load_mut()?;
    let mut liquidatee_orbitlen_account = liquidatee_orbitlen_account_loader.load_mut()?;
    let clock = Clock::get().map_err(|_| OrbitlenError::GetClockFailed)?;
    let current_timestamp = clock.unix_timestamp;
    {
        ctx.accounts.asset_bank.load_mut()?.accrue_interest(current_timestamp)?;
        ctx.accounts.liab_bank.load_mut()?.accrue_interest(current_timestamp)?;
    }

    let (pre_balances, post_balances) = {
        let mut asset_bank = ctx.accounts.asset_bank.load_mut()?;

        let asset_price = {
            let oracle_ais = &ctx.remaining_accounts[0..1];
            fetch_feed_price(&oracle_ais[0], &asset_bank.config)?
        };
        // WIF / USD 2.8
        // let asset_price = 2.8;
        msg!("asset_price: {}", asset_price);

        let mut liab_bank = ctx.accounts.liab_bank.load_mut()?;
        let liab_price = {
            let oracle_ais = &ctx.remaining_accounts[1..2];
            fetch_feed_price(&oracle_ais[0], &liab_bank.config)?
        };

        // let liab_price = 250.0;
        // AAPL / USD 250

        msg!("liab_price: {}", liab_price);

        // Quantity of liability to be paid off by liquidator and received by liquidatee
        let liab_amount = calc_amount(
            calc_value(asset_amount, asset_price, asset_bank.mint_decimals)?,
            liab_price,
            liab_bank.mint_decimals
        )?;

        msg!("liab_amount: {}, asset_amount: {}", liab_amount, asset_amount);

        // Liquidator pays off liability
        let (liquidator_liability_pre_balance, liquidator_liability_post_balance) = {
            let mut bank_account = BankAccountWrapper::find_or_create(
                &ctx.accounts.liab_bank.key(),
                &mut liab_bank,
                &mut liquidator_orbitlen_account.lending_account
            )?;

            let pre_balance = bank_account.bank.get_liability_amount(
                bank_account.balance.liability_shares.into()
            )?;

            bank_account.decrease_balance_in_liquidation(liab_amount)?;

            let post_balance = bank_account.bank.get_liability_amount(
                bank_account.balance.liability_shares.into()
            )?;

            (pre_balance, post_balance)
        };

        msg!(
            "liquidator_liability_pre_balance: {}, liquidator_liability_post_balance: {}",
            liquidator_liability_pre_balance,
            liquidator_liability_post_balance
        );

        // Liquidatee pays off `asset_quantity` amount of collateral
        let (liquidatee_asset_pre_balance, liquidatee_asset_post_balance) = {
            let mut bank_account = BankAccountWrapper::find(
                &ctx.accounts.asset_bank.key(),
                &mut asset_bank,
                &mut liquidatee_orbitlen_account.lending_account
            )?;

            let pre_balance = bank_account.bank.get_asset_amount(
                bank_account.balance.asset_shares.into()
            )?;

            bank_account.withdraw(asset_amount)?;

            let post_balance = bank_account.bank.get_asset_amount(
                bank_account.balance.asset_shares.into()
            )?;

            (pre_balance, post_balance)
        };

        msg!(
            "liquidatee_asset_pre_balance: {}, liquidatee_asset_post_balance: {}",
            liquidatee_asset_pre_balance,
            liquidatee_asset_post_balance
        );

        // Liquidator receives `asset_quantity` amount of collateral
        let (liquidator_asset_pre_balance, liquidator_asset_post_balance) = {
            let mut bank_account = BankAccountWrapper::find_or_create(
                &ctx.accounts.asset_bank.key(),
                &mut asset_bank,
                &mut liquidator_orbitlen_account.lending_account
            )?;

            let pre_balance = bank_account.bank.get_asset_amount(
                bank_account.balance.asset_shares.into()
            )?;

            bank_account.increase_balance_in_liquidation(asset_amount)?;

            let post_balance = bank_account.bank.get_asset_amount(
                bank_account.balance.asset_shares.into()
            )?;

            (pre_balance, post_balance)
        };

        msg!(
            "liquidator_asset_pre_balance: {}, liquidator_asset_post_balance: {}",
            liquidator_asset_pre_balance,
            liquidator_asset_post_balance
        );
        // Liquidatee receives liability payment
        let (liquidatee_liability_pre_balance, liquidatee_liability_post_balance) = {
            let mut liquidatee_liab_bank_account = BankAccountWrapper::find_or_create(
                &ctx.accounts.liab_bank.key(),
                &mut liab_bank,
                &mut liquidatee_orbitlen_account.lending_account
            )?;

            let liquidatee_liability_pre_balance =
                liquidatee_liab_bank_account.bank.get_liability_amount(
                    liquidatee_liab_bank_account.balance.liability_shares.into()
                )?;

            liquidatee_liab_bank_account.increase_balance(liab_amount)?;

            let liquidatee_liability_post_balance =
                liquidatee_liab_bank_account.bank.get_liability_amount(
                    liquidatee_liab_bank_account.balance.liability_shares.into()
                )?;
            (liquidatee_liability_pre_balance, liquidatee_liability_post_balance)
        };
        msg!(
            "liquidatee_liability_pre_balance: {}, liquidatee_liability_post_balance: {}",
            liquidatee_liability_pre_balance,
            liquidatee_liability_post_balance
        );

        (
            LiquidationBalances {
                liquidatee_asset_balance: liquidatee_asset_pre_balance as f64,
                liquidatee_liability_balance: liquidatee_liability_pre_balance as f64,
                liquidator_asset_balance: liquidator_asset_pre_balance as f64,
                liquidator_liability_balance: liquidator_liability_pre_balance as f64,
            },
            LiquidationBalances {
                liquidatee_asset_balance: liquidatee_asset_post_balance as f64,
                liquidatee_liability_balance: liquidatee_liability_post_balance as f64,
                liquidator_asset_balance: liquidator_asset_post_balance as f64,
                liquidator_liability_balance: liquidator_liability_post_balance as f64,
            },
        )
    };

    emit!(LendingAccountLiquidateEvent {
        header: AccountEventHeader {
            signer: ctx.accounts.signer.key(),
            orbitlen_account: liquidator_orbitlen_account_loader.key(),
            orbitlen_account_authority: liquidator_orbitlen_account.authority,
        },
        liquidatee_orbitlen_account: liquidatee_orbitlen_account_loader.key(),
        liquidatee_orbitlen_account_authority: liquidatee_orbitlen_account.authority,
        asset_bank: ctx.accounts.asset_bank.key(),
        asset_mint: ctx.accounts.asset_bank.load_mut()?.mint,
        liability_bank: ctx.accounts.liab_bank.key(),
        liability_mint: ctx.accounts.liab_bank.load_mut()?.mint,
        pre_balances,
        post_balances,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct LendingAccountLiquidate<'info> {
    #[account(mut)]
    pub asset_bank: AccountLoader<'info, Bank>,
    #[account(mut)]
    pub liab_bank: AccountLoader<'info, Bank>,
    #[account(mut)]
    pub liquidator_orbitlen_account: AccountLoader<'info, OrbitlenAccount>,
    #[account(address = liquidator_orbitlen_account.load()?.authority)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub liquidatee_orbitlen_account: AccountLoader<'info, OrbitlenAccount>,
}
