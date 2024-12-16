use anchor_lang::prelude::*;
use crate::error::*;

#[account(zero_copy(unsafe))]
#[derive(Debug, PartialEq, Eq, InitSpace)]
pub struct OrbitlenAccount {
    pub authority: Pubkey,
    pub lending_account: LendingAccount,
}

impl OrbitlenAccount {
    pub fn initialize(&mut self, authority: Pubkey) {
        self.authority = authority;
    }
}

const MAX_LENDING_ACCOUNT_BALANCES: usize = 3;

#[zero_copy(unsafe)]
#[derive(Debug, PartialEq, Eq, InitSpace)]
pub struct LendingAccount {
    pub balances: [Balance; MAX_LENDING_ACCOUNT_BALANCES],
}

impl LendingAccount {
    pub fn get_first_empty_balance(&self) -> Option<usize> {
        self.balances.iter().position(|b| b.bank_pk == Pubkey::default())
    }
}

#[zero_copy(unsafe)]
#[derive(Debug, PartialEq, Eq, InitSpace)]
pub struct Balance {
    pub bank_pk: Pubkey,
    pub asset_shares: i128,
    pub liability_shares: i128,
    pub last_update: u64,
}

impl Balance {
    pub fn change_asset_shares(&mut self, delta: i128) -> Result<()> {
        let asset_shares = self.asset_shares;
        self.asset_shares = asset_shares.checked_add(delta).ok_or(OrbitlenError::MathError)?;
        Ok(())
    }

    pub fn change_liability_shares(&mut self, delta: i128) -> Result<()> {
        let liability_shares = self.liability_shares;
        self.liability_shares = liability_shares
            .checked_add(delta)
            .ok_or(OrbitlenError::MathError)?;
        Ok(())
    }
}