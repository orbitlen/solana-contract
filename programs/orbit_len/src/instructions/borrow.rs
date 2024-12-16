use crate::{ constants::*, error::OrbitlenError, events::*, state::*, utils };
use anchor_lang::prelude::*;
use anchor_spl::token_interface::*;
use solana_program::{ clock::Clock, sysvar::Sysvar };

pub fn lending_account_borrow_process<'info>(
    mut ctx: Context<'_, '_, 'info, 'info, LendingAccountBorrow<'info>>,
    amount: u64
) -> Result<()> {
    let LendingAccountBorrow {
        orbitlen_account: orbitlen_account_loader,
        destination_token_account,
        bank_liquidity_vault,
        token_program,
        bank_liquidity_vault_authority,
        bank: bank_loader,
        ..
    } = ctx.accounts;
    let clock = Clock::get().map_err(|_| OrbitlenError::GetClockFailed)?;

    let maybe_bank_mint = utils::maybe_take_bank_mint(
        &mut ctx.remaining_accounts,
        &*bank_loader.load()?,
        token_program.key
    )?;

    let mut orbitlen_account = orbitlen_account_loader.load_mut()?;

    bank_loader.load_mut()?.accrue_interest(clock.unix_timestamp)?;

    {
        let mut bank = bank_loader.load_mut()?;

        let liquidity_vault_authority_bump = bank.liquidity_vault_authority_bump;

        let mut bank_account = BankAccountWrapper::find_or_create(
            &bank_loader.key(),
            &mut bank,
            &mut orbitlen_account.lending_account
        )?;

        let signer_seeds: &[&[&[u8]]] = &[
            &[
                BankVaultType::Liquidity.get_authority_seed(),
                &bank_loader.key().to_bytes(),
                &[liquidity_vault_authority_bump],
            ],
        ];

        bank_account.borrow(amount.into())?;
        bank_account.withdraw_spl_transfer(
            amount,
            bank_liquidity_vault.to_account_info(),
            destination_token_account.to_account_info(),
            bank_liquidity_vault_authority.to_account_info(),
            maybe_bank_mint.as_ref(),
            token_program.to_account_info(),
            signer_seeds,
            ctx.remaining_accounts
        )?;

        emit!(LendingAccountBorrowEvent {
            header: AccountEventHeader {
                signer: ctx.accounts.signer.key(),
                orbitlen_account: orbitlen_account_loader.key(),
                orbitlen_account_authority: orbitlen_account.authority,
            },
            bank: bank_loader.key(),
            mint: bank.mint,
            amount: amount,
        });
    }

    Ok(())
}

#[derive(Accounts)]
pub struct LendingAccountBorrow<'info> {
    #[account(mut)]
    pub orbitlen_account: AccountLoader<'info, OrbitlenAccount>,
    #[account(address = orbitlen_account.load()?.authority)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub bank: AccountLoader<'info, Bank>,
    #[account(mut)]
    pub destination_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
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
    #[account(
        mut,
        seeds = [
            LIQUIDITY_VAULT_SEED.as_bytes(),
            bank.key().as_ref(),
        ],
        bump = bank.load()?.liquidity_vault_bump,
    )]
    pub bank_liquidity_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
}
