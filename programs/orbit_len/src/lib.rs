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
        ctx: Context<'_, '_, '_, 'info, ProxyDeposit<'info>>,
        coin_amount: u64,
        pc_amount: u64
    ) -> Result<()> {
        raydium::deposit_process(ctx, coin_amount, pc_amount)
    }

    pub fn raydium_withdraw<'info>(
        ctx: Context<'_, '_, '_, 'info, ProxyWithdraw<'info>>,
        amount: u64
    ) -> Result<()> {
        raydium::withdraw_process(ctx, amount)
    }

    pub fn raydium_swap_base_in<'info>(
        ctx: Context<'_, '_, '_, 'info, ProxySwapBaseIn<'info>>,
        amount_in: u64,
        minimum_amount_out: u64
    ) -> Result<()> {
        raydium::swap_base_in_process(ctx, amount_in, minimum_amount_out)
    }

    pub fn raydium_swap_base_out<'info>(
        ctx: Context<'_, '_, '_, 'info, ProxySwapBaseOut<'info>>,
        max_amount_in: u64,
        amount_out: u64
    ) -> Result<()> {
        raydium::swap_base_out_process(ctx, max_amount_in, amount_out)
    }
}
