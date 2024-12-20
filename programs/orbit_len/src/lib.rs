pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;
pub mod events;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;
pub use events::*;

declare_id!("QoB7dVkkZr3oLb95DMpSptvUF8mTygDHNjFQh5y5RAb");

#[program]
pub mod orbit_len {
    use super::*;

    // admin instructions
    pub fn lending_pool_add_bank(
        ctx: Context<LendingPoolAddBank>,
        bank_config: BankConfigCompact
    ) -> Result<()> {
        lending_pool_add_bank_process(ctx, bank_config.into())
    }
    pub fn initial_vault(ctx: Context<InitialVault>, bank: Pubkey) -> Result<()> {
        initial_vault_process(ctx, bank)
    }
    // user instructions
    pub fn initialize_account<'info>(
        ctx: Context<'_, '_, 'info, 'info, OrbitlenAccountInitialize<'info>>
    ) -> Result<()> {
        initialize_account_process(ctx)
    }

    pub fn lending_account_borrow<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountBorrow<'info>>,
        amount: u64
    ) -> Result<()> {
        lending_account_borrow_process(ctx, amount)
    }

    pub fn lending_account_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountDeposit<'info>>,
        amount: u64
    ) -> Result<()> {
        lending_account_deposit_process(ctx, amount)
    }

    pub fn lending_account_liquidate<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountLiquidate<'info>>,
        asset_amount: u64
    ) -> Result<()> {
        lending_account_liquidate_process(ctx, asset_amount)
    }

    // other defi protocols
    pub fn raydium_deposit<'info>(
        ctx: Context<'_, '_, '_, 'info, RaydiumDeposit<'info>>,
        coin_amount: u64,
        pc_amount: u64,
    ) -> Result<()> {
        raydium_deposit_process(ctx, coin_amount, pc_amount)
    }
}
