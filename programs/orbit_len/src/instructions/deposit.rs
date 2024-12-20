use crate::{ constants::*, events::*, state::*, error::*, utils };
use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;

pub fn lending_account_deposit_process<'info>(
    mut ctx: Context<'_, '_, 'info, 'info, LendingAccountDeposit<'info>>,
    amount: u64
) -> Result<()> {
    let LendingAccountDeposit {
        orbitlen_account: orbitlen_account_loader,
        signer,
        signer_token_account,
        bank_liquidity_vault,
        token_program,
        bank: bank_loader,
        ..
    } = ctx.accounts;

    let clock = Clock::get().map_err(|_| OrbitlenError::GetClockFailed)?;
    let maybe_bank_mint = utils::maybe_take_bank_mint(
        &mut ctx.remaining_accounts,
        &*bank_loader.load()?,
        token_program.key
    )?;

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

    bank_account.deposit(amount.into())?;

    bank_account.deposit_spl_transfer(
        amount,
        signer_token_account.to_account_info(),
        bank_liquidity_vault.to_account_info(),
        signer.to_account_info(),
        &maybe_bank_mint,
        token_program.to_account_info(),
        ctx.remaining_accounts
    )?;

    emit!(LendingAccountDepositEvent {
        header: AccountEventHeader {
            signer: signer.key(),
            orbitlen_account: orbitlen_account_loader.key(),
            orbitlen_account_authority: orbitlen_account.authority,
        },
        bank: bank_loader.key(),
        mint: bank.mint,
        amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct LendingAccountDeposit<'info> {
    #[account(
        mut,
        seeds = [ORBITLEN_ACCOUNT_SEED.as_bytes(), signer.key().as_ref()],
        bump,
    )]
    pub orbitlen_account: AccountLoader<'info, OrbitlenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub bank: AccountLoader<'info, Bank>,
    /// CHECK: Token mint/authority are checked at transfer
    #[account(mut)]
    pub signer_token_account: AccountInfo<'info>,
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
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
