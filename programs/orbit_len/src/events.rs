use anchor_lang::prelude::*;

#[event]
pub struct LendingAccountLiquidateEvent {
    pub header: AccountEventHeader,
    pub liquidatee_orbitlen_account: Pubkey,
    pub liquidatee_orbitlen_account_authority: Pubkey,
    pub asset_bank: Pubkey,
    pub asset_mint: Pubkey,
    pub liability_bank: Pubkey,
    pub liability_mint: Pubkey,
    pub pre_balances: LiquidationBalances,
    pub post_balances: LiquidationBalances,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct LiquidationBalances {
    pub liquidatee_asset_balance: f64,
    pub liquidatee_liability_balance: f64,
    pub liquidator_asset_balance: f64,
    pub liquidator_liability_balance: f64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AccountEventHeader {
    pub signer: Pubkey,
    pub orbitlen_account: Pubkey,
    pub orbitlen_account_authority: Pubkey,
}

#[event]
pub struct LendingPoolBankCreateEvent {
    pub signer: Pubkey,
    pub bank: Pubkey,
    pub mint: Pubkey,
}

#[event]
pub struct OrbitlenAccountCreateEvent {
    pub header: AccountEventHeader,
}

#[event]
pub struct LendingAccountDepositEvent {
    pub header: AccountEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

#[event]
pub struct LendingAccountBorrowEvent {
    pub header: AccountEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

/**
 * Raydium events
 */

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RaydiumEventHeader {
    pub amm: Pubkey,
    pub market: Pubkey,
    pub signer: Pubkey,
    pub orbitlen_account: Pubkey,
    pub orbitlen_account_authority: Pubkey,
}

#[event]
pub struct RaydiumDepositEvent {
    pub header: RaydiumEventHeader,
    pub coin_mint: Pubkey,
    pub coin_amount: u64,
    pub pc_amount: u64,
}
