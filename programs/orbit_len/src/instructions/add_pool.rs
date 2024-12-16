use crate::{ constants::*, events::*, state::* };
use anchor_lang::prelude::*;
use anchor_spl::token_interface::*;

pub fn lending_pool_add_bank(
    ctx: Context<LendingPoolAddBank>,
    bank_config: BankConfig
) -> Result<()> {
    msg!("Adding bank to lending pool");
    let LendingPoolAddBank {
        bank_mint,
        liquidity_vault,
        insurance_vault,
        bank: bank_loader,
        ..
    } = ctx.accounts;

    let mut bank = bank_loader.load_init()?;
    msg!("Bank config: {:?}", bank);

    let liquidity_vault_bump = ctx.bumps.liquidity_vault;
    let liquidity_vault_authority_bump = ctx.bumps.liquidity_vault_authority;
    let insurance_vault_bump = ctx.bumps.insurance_vault;
    let insurance_vault_authority_bump = ctx.bumps.insurance_vault_authority;

    *bank = Bank::new(
        bank_mint.key(),
        bank_mint.decimals,
        bank_config,
        Clock::get().unwrap().unix_timestamp,

        liquidity_vault.key(),
        insurance_vault.key(),
        liquidity_vault_bump,
        liquidity_vault_authority_bump,
        insurance_vault_bump,
        insurance_vault_authority_bump
    );

    emit!(LendingPoolBankCreateEvent {
        signer: *ctx.accounts.admin.key,
        bank: bank_loader.key(),
        mint: bank_mint.key(),
    });

    Ok(())
}

#[derive(Accounts)]
pub struct LendingPoolAddBank<'info> {
    #[account(
        mut,
    )]
    pub admin: Signer<'info>,

    pub bank_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(init, space = 8 + std::mem::size_of::<Bank>(), payer = admin)]
    pub bank: AccountLoader<'info, Bank>,

    /// CHECK: ⋐ ͡⋄ ω ͡⋄ ⋑
    #[account(seeds = [LIQUIDITY_VAULT_AUTHORITY_SEED.as_bytes(), bank.key().as_ref()], bump)]
    pub liquidity_vault_authority: AccountInfo<'info>,

    #[account(
        mut,
        token::mint = bank_mint,
        token::authority = liquidity_vault_authority,
        seeds = [LIQUIDITY_VAULT_SEED.as_bytes(), bank.key().as_ref()],
        bump
    )]
    pub liquidity_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: ⋐ ͡⋄ ω ͡⋄ ⋑
    #[account(seeds = [INSURANCE_VAULT_AUTHORITY_SEED.as_bytes(), bank.key().as_ref()], bump)]
    pub insurance_vault_authority: AccountInfo<'info>,

    #[account(
        mut,
        token::mint = bank_mint,
        token::authority = insurance_vault_authority,
        seeds = [INSURANCE_VAULT_SEED.as_bytes(), bank.key().as_ref()],
        bump
    )]
    pub insurance_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn initial_vault(
    ctx: Context<InitialVault>,
    bank: Pubkey
) -> Result<()> {
    msg!("Initial Vault");

    Ok(())
}

#[derive(Accounts)]
#[instruction(bank: Pubkey)]
pub struct InitialVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub bank_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: ⋐ ͡⋄ ω ͡⋄ ⋑
    #[account(seeds = [LIQUIDITY_VAULT_AUTHORITY_SEED.as_bytes(), bank.key().as_ref()], bump)]
    pub liquidity_vault_authority: AccountInfo<'info>,
    #[account(
        init,
        payer = admin,
        token::mint = bank_mint,
        token::authority = liquidity_vault_authority,
        seeds = [LIQUIDITY_VAULT_SEED.as_bytes(), bank.key().as_ref()],
        bump
    )]
    pub liquidity_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: ⋐ ͡⋄ ω ͡⋄ ⋑
    #[account(seeds = [INSURANCE_VAULT_AUTHORITY_SEED.as_bytes(), bank.key().as_ref()], bump)]
    pub insurance_vault_authority: AccountInfo<'info>,
    #[account(
        init,
        payer = admin,
        token::mint = bank_mint,
        token::authority = insurance_vault_authority,
        seeds = [INSURANCE_VAULT_SEED.as_bytes(), bank.key().as_ref()],
        bump
    )]
    pub insurance_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}



