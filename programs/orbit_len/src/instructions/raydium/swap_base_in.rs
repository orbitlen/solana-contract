use anchor_lang::prelude::*;
use anchor_spl::{
    token::{ Token, ID as TOKEN_PROGRAM_ID },
    token_interface::{ Mint, TokenAccount },
};
use raydium_amm_cpi::SwapBaseIn;
use crate::{ constants::*, bank::*, account::*, error::*, events::* };

#[derive(Accounts, Clone)]
pub struct ProxySwapBaseIn<'info> {
    /**
     * Raydium accounts
     */
    /// CHECK: Safe
    pub amm_program: UncheckedAccount<'info>,
    /// CHECK: Safe. amm Account
    #[account(mut)]
    pub amm: UncheckedAccount<'info>,
    /// CHECK: Safe. Amm authority Account
    #[account()]
    pub amm_authority: UncheckedAccount<'info>,
    /// CHECK: Safe. amm open_orders Account
    #[account(mut)]
    pub amm_open_orders: UncheckedAccount<'info>,
    /// CHECK: Safe. amm_coin_vault Amm Account to swap FROM or To,
    #[account(mut)]
    pub amm_coin_vault: UncheckedAccount<'info>,
    /// CHECK: Safe. amm_pc_vault Amm Account to swap FROM or To,
    #[account(mut)]
    pub amm_pc_vault: UncheckedAccount<'info>,
    /// CHECK: Safe.OpenBook program id
    pub market_program: UncheckedAccount<'info>,
    /// CHECK: Safe. OpenBook market Account. OpenBook program is the owner.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: Safe. bids Account
    #[account(mut)]
    pub market_bids: UncheckedAccount<'info>,
    /// CHECK: Safe. asks Account
    #[account(mut)]
    pub market_asks: UncheckedAccount<'info>,
    /// CHECK: Safe. event_q Account
    #[account(mut)]
    pub market_event_queue: UncheckedAccount<'info>,
    /// CHECK: Safe. coin_vault Account
    #[account(mut)]
    pub market_coin_vault: UncheckedAccount<'info>,
    /// CHECK: Safe. pc_vault Account
    #[account(mut)]
    pub market_pc_vault: UncheckedAccount<'info>,
    /// CHECK: Safe. vault_signer Account
    #[account(mut)]
    pub market_vault_signer: UncheckedAccount<'info>,
    /// CHECK: Safe. user source token Account. user Account to swap from.
    #[account(mut)]
    pub user_token_source: UncheckedAccount<'info>,
    /// CHECK: Safe. user destination token Account. user Account to swap to.
    #[account(mut)]
    pub user_token_destination: UncheckedAccount<'info>,
    /// CHECK: Safe. user owner Account
    #[account(mut)]
    pub user_source_owner: Signer<'info>,
    /**
     * Orbitlen accounts
     */
    #[account(
        mut,
        seeds = [ORBITLEN_ACCOUNT_SEED.as_bytes(), user_source_owner.key().as_ref()],
        bump,
    )]
    pub orbitlen_account: AccountLoader<'info, OrbitlenAccount>,
    #[account(
        mut,        
        seeds = [BANK_SEED.as_bytes(), coin_mint.key().as_ref()],
        bump
    )]
    pub bank: AccountLoader<'info, Bank>,
    /// CHECK: Seed constraint check
    #[account(
        mut,
        seeds = [
            LIQUIDITY_VAULT_SEED.as_bytes(),
            bank.key().as_ref(),
        ],
        bump = bank.load()?.liquidity_vault_bump,
    )]
    pub bank_liquidity_vault: AccountInfo<'info>,
    /// CHECK: Seed constraint check
    #[account(
            mut,
            seeds = [
                LIQUIDITY_VAULT_AUTHORITY_SEED.as_bytes(),
                bank.key().as_ref(),
            ],
            bump = bank.load()?.liquidity_vault_authority_bump,
        )]
    pub bank_liquidity_vault_authority: AccountInfo<'info>,
    #[account(mut)]
    pub user_coin_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mint::decimals = COMMON_TOKEN_DECIMALS)]
    pub coin_mint: InterfaceAccount<'info, Mint>,
    #[account(address = TOKEN_PROGRAM_ID)]
    pub token_program: Program<'info, Token>,
}

pub fn swap_base_in_process<'info>(
    ctx: Context<'_, '_, '_, 'info, ProxySwapBaseIn<'info>>,
    amount_in: u64,
    minimum_amount_out: u64
) -> Result<()> {
    let accounts = ctx.accounts;
    let coin_amount_before = accounts.user_coin_token_account.amount;
    msg!("coin_amount_before: {}", coin_amount_before);

    let cpi_accounts = SwapBaseIn {
        amm: accounts.amm.clone(),
        amm_authority: accounts.amm_authority.clone(),
        amm_open_orders: accounts.amm_open_orders.clone(),
        amm_coin_vault: accounts.amm_coin_vault.clone(),
        amm_pc_vault: accounts.amm_pc_vault.clone(),
        market_program: accounts.market_program.clone(),
        market: accounts.market.clone(),
        market_bids: accounts.market_bids.clone(),
        market_asks: accounts.market_asks.clone(),
        market_event_queue: accounts.market_event_queue.clone(),
        market_coin_vault: accounts.market_coin_vault.clone(),
        market_pc_vault: accounts.market_pc_vault.clone(),
        market_vault_signer: accounts.market_vault_signer.clone(),
        user_token_source: accounts.user_token_source.clone(),
        user_token_destination: accounts.user_token_destination.clone(),
        user_source_owner: accounts.user_source_owner.clone(),
        token_program: accounts.token_program.clone(),
    };
    let cpi_program = accounts.amm_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    raydium_amm_cpi::swap_base_in(cpi_ctx, amount_in, minimum_amount_out)?;

    accounts.user_coin_token_account.reload()?;
    let coin_amount_after = accounts.user_coin_token_account.amount;

    msg!("coin_amount_after: {}", coin_amount_after);

    let ProxySwapBaseIn {
        orbitlen_account: orbitlen_account_loader,
        user_source_owner,
        user_coin_token_account,
        bank_liquidity_vault,
        bank_liquidity_vault_authority,
        token_program,
        bank: bank_loader,
        coin_mint,
        ..
    } = accounts;

    let clock = Clock::get().map_err(|_| OrbitlenError::GetClockFailed)?;

    let mut bank = bank_loader.load_mut()?;
    msg!("bank: {:?}", bank);
    let mut orbitlen_account = orbitlen_account_loader.load_mut()?;
    msg!("orbitlen_account: {:?}", orbitlen_account);
    bank.accrue_interest(clock.unix_timestamp)?;
    let liquidity_vault_authority_bump = bank.liquidity_vault_authority_bump;

    let mut bank_account = BankAccountWrapper::find_or_create(
        &bank_loader.key(),
        &mut bank,
        &mut orbitlen_account.lending_account
    )?;

    let is_increase_coin = coin_amount_after > coin_amount_before;

    let coin_amount_delta = u64::abs_diff(coin_amount_after, coin_amount_before);

    msg!("is_increase_coin: {}, coin_amount_delta:{}", is_increase_coin, coin_amount_delta);

    if is_increase_coin {
        bank_account.deposit(coin_amount_delta.into())?;

        bank_account.deposit_spl_transfer(
            coin_amount_delta,
            user_coin_token_account.to_account_info(),
            bank_liquidity_vault.to_account_info(),
            user_source_owner.to_account_info(),
            coin_mint,
            token_program.to_account_info(),
            ctx.remaining_accounts
        )?;

        emit!(LendingAccountDepositEvent {
            header: AccountEventHeader {
                signer: user_source_owner.key(),
                orbitlen_account: orbitlen_account_loader.key(),
                orbitlen_account_authority: orbitlen_account.authority,
            },
            bank: bank_loader.key(),
            mint: bank.mint,
            amount: coin_amount_delta,
        });
    } else {
        let signer_seeds: &[&[&[u8]]] = &[
            &[
                BankVaultType::Liquidity.get_authority_seed(),
                &bank_loader.key().to_bytes(),
                &[liquidity_vault_authority_bump],
            ],
        ];

        bank_account.withdraw(coin_amount_delta.into())?;
        bank_account.withdraw_spl_transfer(
            coin_amount_delta,
            bank_liquidity_vault.to_account_info(),
            user_coin_token_account.to_account_info(),
            bank_liquidity_vault_authority.to_account_info(),
            coin_mint,
            token_program.to_account_info(),
            signer_seeds,
            ctx.remaining_accounts
        )?;

        emit!(LendingAccountBorrowEvent {
            header: AccountEventHeader {
                signer: user_source_owner.key(),
                orbitlen_account: orbitlen_account_loader.key(),
                orbitlen_account_authority: orbitlen_account.authority,
            },
            bank: bank_loader.key(),
            mint: bank.mint,
            amount: coin_amount_delta,
        });
    }

    Ok(())
}
