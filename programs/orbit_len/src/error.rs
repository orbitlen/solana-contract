use anchor_lang::prelude::*;

#[error_code]
pub enum OrbitlenError {
    #[msg("Clock error")]
    GetClockFailed,
    #[msg("Math error on compute")]
    MathError,
    #[msg("Invalid transfer")]
    InvalidTransfer,
    #[msg("Token22 Banks require mint account as first remaining account")]
    T22MintRequired,
    #[msg("Lending account balance slots are full")]
    LendingAccountBalanceSlotsFull,
    #[msg("Bank is missing")]
    BankAccountNotFound,
    #[msg("Invalid bank utilization ratio")]
    IllegalUtilizationRatio,
    #[msg("Illegal liquidation")]
    IllegalLiquidation,
    #[msg("fetch price failed")]
    FetchPriceFailed,
    #[msg("Invalid price feed pubkey")]
    InvalidPriceFeedPk,
    #[msg("Math error on interest rate config")]
    InterestRateConfigMathError,
}
