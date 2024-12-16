use anchor_lang::prelude::*;


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
