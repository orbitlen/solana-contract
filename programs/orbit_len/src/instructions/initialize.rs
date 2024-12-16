use crate::{ events::*, state::*, constants::* };
use anchor_lang::prelude::*;

pub fn initialize_account_process(ctx: Context<OrbitlenAccountInitialize>) -> Result<()> {
    let OrbitlenAccountInitialize {
        authority,
        orbitlen_account: orbitlen_account_loader,
        ..
    } = ctx.accounts;

    let mut orbitlen_account = orbitlen_account_loader.load_init()?;

    orbitlen_account.initialize(authority.key());

    emit!(OrbitlenAccountCreateEvent {
        header: AccountEventHeader {
            signer: authority.key(),
            orbitlen_account: orbitlen_account_loader.key(),
            orbitlen_account_authority: orbitlen_account.authority,
        },
    });

    Ok(())
}

#[derive(Accounts)]
pub struct OrbitlenAccountInitialize<'info> {
    #[account(
        init,
        seeds = [ORBITLEN_ACCOUNT_SEED.as_bytes(), authority.key().as_ref()],
        bump,
        payer = authority,
        space = 8 + OrbitlenAccount::INIT_SPACE
    )]
    pub orbitlen_account: AccountLoader<'info, OrbitlenAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
