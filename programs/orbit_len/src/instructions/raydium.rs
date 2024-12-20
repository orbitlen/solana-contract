use anchor_lang::prelude::*;
use anchor_spl::{ token::Token, token_interface::{ Mint, TokenAccount } };
use raydium_amm_cpi::Deposit;
use crate::{ constants::*, bank::*, account::*, error::*, events::* };

#[derive(Accounts, Clone)]
pub struct RaydiumDeposit<'info> {
    /// CHECK: Safe
    pub amm_program: UncheckedAccount<'info>,
    /// CHECK: Safe. Amm Account
    #[account(mut)]
    pub amm: UncheckedAccount<'info>,
    /// CHECK: Safe. Amm authority, a PDA create with seed = [b"amm authority"]
    #[account()]
    pub amm_authority: UncheckedAccount<'info>,
    /// CHECK: Safe. AMM open_orders Account.
    #[account()]
    pub amm_open_orders: UncheckedAccount<'info>,
    /// CHECK: Safe. AMM target orders account. To store plan orders infomations.
    #[account(mut)]
    pub amm_target_orders: UncheckedAccount<'info>,
    /// CHECK: Safe. LP mint account. Must be empty, owned by $authority.
    #[account(mut)]
    pub amm_lp_mint: UncheckedAccount<'info>,
    /// CHECK: Safe. amm_coin_vault account, $authority can transfer amount.
    #[account(mut)]
    pub amm_coin_vault: UncheckedAccount<'info>,
    /// CHECK: Safe. amm_pc_vault account, $authority can transfer amount.
    #[account(mut)]
    pub amm_pc_vault: UncheckedAccount<'info>,
    /// CHECK: Safe. OpenBook market account, OpenBook program is the owner.
    pub market: UncheckedAccount<'info>,
    /// CHECK: Safe. OpenBook market event queue account, OpenBook program is the owner.
    pub market_event_queue: UncheckedAccount<'info>,
    /// CHECK: Safe. User token coin to deposit into.
    #[account(mut)]
    pub user_token_coin: UncheckedAccount<'info>,
    /// CHECK: Safe. User token pc to deposit into.
    #[account(mut)]
    pub user_token_pc: UncheckedAccount<'info>,
    /// CHECK: Safe. User lp token, to deposit the generated tokens, user is the owner
    #[account(mut)]
    pub user_token_lp: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [ORBITLEN_ACCOUNT_SEED.as_bytes(), user_owner.key().as_ref()],
        bump,
    )]
    pub orbitlen_account: AccountLoader<'info, OrbitlenAccount>,
    #[account(mut)]
    pub coin_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [BANK_SEED.as_bytes(), coin_mint.key().as_ref()],
        bump
    )]
    pub coin_bank: AccountLoader<'info, Bank>,
    /// CHECK: Seed constraint check
    #[account(
        mut,
        seeds = [
            LIQUIDITY_VAULT_SEED.as_bytes(),
            coin_bank.key().as_ref(),
        ],
        bump = coin_bank.load()?.liquidity_vault_bump,
    )]
    pub coin_bank_liquidity_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: Seed constraint check
    #[account(
        mut,
        seeds = [
            LIQUIDITY_VAULT_AUTHORITY_SEED.as_bytes(),
            coin_bank.key().as_ref(),
        ],
        bump = coin_bank.load()?.liquidity_vault_authority_bump,
    )]
    pub coin_bank_liquidity_vault_authority: AccountInfo<'info>,
    /// CHECK: Safe. User wallet account
    #[account(mut)]
    pub user_owner: Signer<'info>,
    /// CHECK: Safe. The spl token program
    pub token_program: Program<'info, Token>,
}

pub fn raydium_deposit_process<'info>(
    ctx: Context<'_, '_, '_, 'info, RaydiumDeposit<'info>>,
    coin_amount: u64,
    pc_amount: u64
) -> Result<()> {
    let RaydiumDeposit {
        orbitlen_account: orbitlen_account_loader,
        coin_bank_liquidity_vault,
        token_program,
        coin_bank: coin_bank_loader,
        coin_mint,
        user_token_coin,
        coin_bank_liquidity_vault_authority,
        amm,
        market,
        user_owner,
        ..
    } = ctx.accounts;

    {
        let clock = Clock::get().map_err(|_| OrbitlenError::GetClockFailed)?;

        let mut coin_bank = coin_bank_loader.load_mut()?;
        msg!("coin_bank: {:?}", coin_bank);

        let coin_liquidity_vault_authority_bump = coin_bank.liquidity_vault_authority_bump;

        let mut orbitlen_account = orbitlen_account_loader.load_mut()?;
        msg!("orbitlen_account: {:?}", orbitlen_account);

        coin_bank.accrue_interest(clock.unix_timestamp)?;

        let mut coin_bank_account = BankAccountWrapper::find_or_create(
            &coin_bank_loader.key(),
            &mut coin_bank,
            &mut orbitlen_account.lending_account
        )?;

        coin_bank_account.decrease_balance(coin_amount)?;

        //coin vault --> user coin token account
        let signer_seeds: &[&[&[u8]]] = &[
            &[
                BankVaultType::Liquidity.get_authority_seed(),
                &coin_bank_loader.key().to_bytes(),
                &[coin_liquidity_vault_authority_bump],
            ],
        ];

        coin_bank_account.withdraw_spl_transfer(
            coin_amount,
            coin_bank_liquidity_vault.to_account_info(),
            user_token_coin.to_account_info(),
            coin_bank_liquidity_vault_authority.to_account_info(),
            coin_mint,
            token_program.to_account_info(),
            signer_seeds,
            ctx.remaining_accounts
        )?;
    }

    let orbitlen_account_key = orbitlen_account_loader.key();
    let orbitlen_account_authority = orbitlen_account_loader.load()?.authority;
    let coin_mint_key = coin_mint.key();
    let amm_key = *amm.key;
    let market_key = *market.key;
    let signer_key = *user_owner.key;

    {
        // user coin token account --> coin vault in raydium amm pool
        let accounts = ctx.accounts;
        let cpi_accounts = Deposit {
            amm: accounts.amm.clone(),
            amm_authority: accounts.amm_authority.clone(),
            amm_open_orders: accounts.amm_open_orders.clone(),
            amm_target_orders: accounts.amm_target_orders.clone(),
            amm_lp_mint: accounts.amm_lp_mint.clone(),
            amm_coin_vault: accounts.amm_coin_vault.clone(),
            amm_pc_vault: accounts.amm_pc_vault.clone(),
            market: accounts.market.clone(),
            market_event_queue: accounts.market_event_queue.clone(),
            user_token_coin: accounts.user_token_coin.clone(),
            user_token_pc: accounts.user_token_pc.clone(),
            user_token_lp: accounts.user_token_lp.clone(),
            user_owner: accounts.user_owner.clone(),
            token_program: accounts.token_program.clone(),
        };
        let cpi_program = accounts.amm_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        raydium_amm_cpi::deposit(cpi_ctx, coin_amount, pc_amount, 0)?;
    }

    emit!(RaydiumDepositEvent {
        header: RaydiumEventHeader {
            amm: amm_key,
            market: market_key,
            signer: signer_key,
            orbitlen_account: orbitlen_account_key,
            orbitlen_account_authority: orbitlen_account_authority,
        },
        coin_mint: coin_mint_key,
        coin_amount,
        pc_amount,
    });

    Ok(())
}
