use anchor_lang::prelude::*;
use anchor_spl::{
    token::{ Token, ID as TOKEN_PROGRAM_ID },
    token_interface::{ Mint, TokenAccount },
};
use raydium_amm_cpi::Withdraw;
use crate::{ constants::*, bank::*, account::*, error::*, events::* };

#[derive(Accounts, Clone)]
pub struct ProxyWithdraw<'info> {
    /**
     * Raydium accounts
     */
    /// CHECK: Safe
    pub amm_program: UncheckedAccount<'info>,
    /// CHECK: Safe. Amm account
    #[account(mut)]
    pub amm: UncheckedAccount<'info>,
    /// CHECK: Safe. Amm authority Account
    #[account()]
    pub amm_authority: UncheckedAccount<'info>,
    /// CHECK: Safe. amm open_orders Account
    #[account(mut)]
    pub amm_open_orders: UncheckedAccount<'info>,
    /// CHECK: Safe. amm target_orders Account. To store plan orders infomations.
    #[account(mut)]
    pub amm_target_orders: UncheckedAccount<'info>,
    /// CHECK: Safe. pool lp mint account. Must be empty, owned by $authority.
    #[account(mut)]
    pub amm_lp_mint: UncheckedAccount<'info>,
    /// CHECK: Safe. amm_coin_vault Amm Account to withdraw FROM,
    #[account(mut)]
    pub amm_coin_vault: UncheckedAccount<'info>,
    /// CHECK: Safe. amm_pc_vault Amm Account to withdraw FROM,
    #[account(mut)]
    pub amm_pc_vault: UncheckedAccount<'info>,
    /// CHECK: Safe. OpenBook program id
    pub market_program: UncheckedAccount<'info>,
    /// CHECK: Safe. OpenBook market Account. OpenBook program is the owner.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: Safe. OpenBook coin_vault Account
    #[account(mut)]
    pub market_coin_vault: UncheckedAccount<'info>,
    /// CHECK: Safe. OpenBook pc_vault Account
    #[account(mut)]
    pub market_pc_vault: UncheckedAccount<'info>,
    /// CHECK: Safe. OpenBook vault_signer Account
    pub market_vault_signer: UncheckedAccount<'info>,
    /// CHECK: Safe. user lp token Account. Source lp, amount is transferable by $authority.
    #[account(mut)]
    pub user_token_lp: UncheckedAccount<'info>,
    /// CHECK: Safe. user token coin Account. user Account to credit.
    #[account(mut)]
    pub user_token_coin: UncheckedAccount<'info>,
    /// CHECK: Safe. user token pc Account. user Account to credit.
    #[account(mut)]
    pub user_token_pc: UncheckedAccount<'info>,
    /// CHECK: Safe. User wallet account
    #[account(mut)]
    pub user_owner: Signer<'info>,
    /// CHECK: Safe. OpenBook event queue account
    #[account(mut)]
    pub market_event_q: UncheckedAccount<'info>,
    /// CHECK: Safe. OpenBook bid account
    #[account(mut)]
    pub market_bids: UncheckedAccount<'info>,
    /// CHECK: Safe. OpenBook ask account
    #[account(mut)]
    pub market_asks: UncheckedAccount<'info>,
    /**
     * Orbitlen accounts
     */
    #[account(
        mut,
        seeds = [ORBITLEN_ACCOUNT_SEED.as_bytes(), user_owner.key().as_ref()],
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
    #[account(mut)]
    pub user_coin_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mint::decimals = COMMON_TOKEN_DECIMALS)]
    pub coin_mint: InterfaceAccount<'info, Mint>,
    #[account(address = TOKEN_PROGRAM_ID)]
    pub token_program: Program<'info, Token>,
}

pub fn withdraw_process<'info>(
    ctx: Context<'_, '_, '_, 'info, ProxyWithdraw<'info>>,
    amount: u64
) -> Result<()> {
    let accounts = ctx.accounts;

    let coin_amount_before = accounts.user_coin_token_account.amount;
    msg!("coin_amount_before: {}", coin_amount_before);

    let cpi_accounts = Withdraw {
        amm: accounts.amm.clone(),
        amm_authority: accounts.amm_authority.clone(),
        amm_open_orders: accounts.amm_open_orders.clone(),
        amm_target_orders: accounts.amm_target_orders.clone(),
        amm_lp_mint: accounts.amm_lp_mint.clone(),
        amm_coin_vault: accounts.amm_coin_vault.clone(),
        amm_pc_vault: accounts.amm_pc_vault.clone(),
        market_program: accounts.market_program.clone(),
        market: accounts.market.clone(),
        market_coin_vault: accounts.market_coin_vault.clone(),
        market_pc_vault: accounts.market_pc_vault.clone(),
        market_vault_signer: accounts.market_vault_signer.clone(),
        user_token_lp: accounts.user_token_lp.clone(),
        user_token_coin: accounts.user_token_coin.clone(),
        user_token_pc: accounts.user_token_pc.clone(),
        user_owner: accounts.user_owner.clone(),
        market_event_q: accounts.market_event_q.clone(),
        market_bids: accounts.market_bids.clone(),
        market_asks: accounts.market_asks.clone(),
        token_program: accounts.token_program.clone(),
    };
    let cpi_program = accounts.amm_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    raydium_amm_cpi::withdraw(cpi_ctx, amount)?;

    accounts.user_coin_token_account.reload()?;
    let coin_amount_after = accounts.user_coin_token_account.amount;

    msg!("coin_amount_after: {}", coin_amount_after);

    let deposit_amount = coin_amount_after - coin_amount_before;

    msg!("transfer amount:{}", deposit_amount);

    let ProxyWithdraw {
        orbitlen_account: orbitlen_account_loader,
        user_owner,
        user_coin_token_account,
        bank_liquidity_vault,
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

    let mut bank_account = BankAccountWrapper::find_or_create(
        &bank_loader.key(),
        &mut bank,
        &mut orbitlen_account.lending_account
    )?;

    bank_account.deposit(deposit_amount.into())?;

    bank_account.deposit_spl_transfer(
        deposit_amount,
        user_coin_token_account.to_account_info(),
        bank_liquidity_vault.to_account_info(),
        user_owner.to_account_info(),
        coin_mint,
        token_program.to_account_info(),
        ctx.remaining_accounts
    )?;

    emit!(LendingAccountDepositEvent {
        header: AccountEventHeader {
            signer: user_owner.key(),
            orbitlen_account: orbitlen_account_loader.key(),
            orbitlen_account_authority: orbitlen_account.authority,
        },
        bank: bank_loader.key(),
        mint: bank.mint,
        amount: deposit_amount,
    });

    Ok(())
}
