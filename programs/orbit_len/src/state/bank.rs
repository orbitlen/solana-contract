use anchor_lang::prelude::*;
use crate::{ state::account::*, error::*, constants::* };
use std::cmp::{ max, min };
use anchor_spl::token_interface::*;

#[account(zero_copy(unsafe))]
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Bank {
    pub mint: Pubkey,
    pub mint_decimals: u8,

    pub asset_share_value: i128,

    pub liability_share_value: i128,

    pub liquidity_vault: Pubkey,
    pub liquidity_vault_bump: u8,
    pub liquidity_vault_authority_bump: u8,

    pub insurance_vault: Pubkey,
    pub insurance_vault_bump: u8,
    pub insurance_vault_authority_bump: u8,

    pub total_liability_shares: i128,
    pub total_asset_shares: i128,

    pub last_update: i64,

    pub config: BankConfig,
}

impl Bank {
    pub fn new(
        mint: Pubkey,
        mint_decimals: u8,
        config: BankConfig,
        current_timestamp: i64,
        liquidity_vault: Pubkey,
        insurance_vault: Pubkey,
        liquidity_vault_bump: u8,
        liquidity_vault_authority_bump: u8,
        insurance_vault_bump: u8,
        insurance_vault_authority_bump: u8
    ) -> Bank {
        Bank {
            mint,
            mint_decimals,
            asset_share_value: 1,
            liability_share_value: 1,
            liquidity_vault,
            liquidity_vault_bump,
            liquidity_vault_authority_bump,
            insurance_vault,
            insurance_vault_bump,
            insurance_vault_authority_bump,
            total_liability_shares: 0,
            total_asset_shares: 0,
            last_update: current_timestamp,
            config,
            ..Default::default()
        }
    }

    pub fn deposit_spl_transfer<'info>(
        &self,
        amount: u64,
        from: AccountInfo<'info>,
        to: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        maybe_mint: Option<&InterfaceAccount<'info, Mint>>,
        program: AccountInfo<'info>,
        remaining_accounts: &[AccountInfo<'info>]
    ) -> Result<()> {
        require_keys_eq!(*to.key, self.liquidity_vault, OrbitlenError::InvalidTransfer);

        msg!(
            "deposit_spl_transfer: amount: {} from {} to {}, auth {}",
            amount,
            from.key,
            to.key,
            authority.key
        );

        if let Some(mint) = maybe_mint {
            spl_token_2022::onchain::invoke_transfer_checked(
                program.key,
                from,
                mint.to_account_info(),
                to,
                authority,
                remaining_accounts,
                amount,
                mint.decimals,
                &[]
            )?;
        } else {
            #[allow(deprecated)]
            transfer(
                CpiContext::new_with_signer(
                    program,
                    Transfer {
                        from,
                        to,
                        authority,
                    },
                    &[]
                ),
                amount
            )?;
        }

        Ok(())
    }

    pub fn withdraw_spl_transfer<'info>(
        &self,
        amount: u64,
        from: AccountInfo<'info>,
        to: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        maybe_mint: Option<&InterfaceAccount<'info, Mint>>,
        program: AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
        remaining_accounts: &[AccountInfo<'info>]
    ) -> Result<()> {
        msg!(
            "withdraw_spl_transfer: amount: {} from {} to {}, auth {}",
            amount,
            from.key,
            to.key,
            authority.key
        );

        if let Some(mint) = maybe_mint {
            spl_token_2022::onchain::invoke_transfer_checked(
                program.key,
                from,
                mint.to_account_info(),
                to,
                authority,
                remaining_accounts,
                amount,
                mint.decimals,
                signer_seeds
            )?;
        } else {
            #[allow(deprecated)]
            transfer(
                CpiContext::new_with_signer(
                    program,
                    Transfer {
                        from,
                        to,
                        authority,
                    },
                    signer_seeds
                ),
                amount
            )?;
        }

        Ok(())
    }

    pub fn check_utilization_ratio(&self) -> Result<()> {
        let total_assets = self.get_asset_amount(self.total_asset_shares.into())?;
        let total_liabilities = self.get_liability_amount(self.total_liability_shares.into())?;

        require_gte!(total_assets, total_liabilities, OrbitlenError::IllegalUtilizationRatio);

        Ok(())
    }

    pub fn get_liability_amount(&self, shares: i128) -> Result<i128> {
        shares.checked_mul(self.liability_share_value.into()).ok_or(OrbitlenError::MathError.into())
    }

    pub fn get_asset_amount(&self, shares: i128) -> Result<i128> {
        shares.checked_mul(self.asset_share_value.into()).ok_or(OrbitlenError::MathError.into())
    }

    pub fn accrue_interest(&mut self, current_timestamp: i64) -> Result<()> {
        let time_delta: u64 = (current_timestamp - self.last_update).try_into().unwrap();

        if time_delta == 0 {
            return Ok(());
        }

        let total_assets = self.get_asset_amount(self.total_asset_shares.into())?;
        let total_liabilities = self.get_liability_amount(self.total_liability_shares.into())?;

        self.last_update = current_timestamp;

        if total_assets == 0 || total_liabilities == 0 {
            return Ok(());
        }

        let (asset_share_value, liability_share_value) = calc_interest_rate_accrual_state_changes(
            time_delta,
            total_assets,
            total_liabilities,
            &self.config.interest_rate_config,
            self.asset_share_value.into(),
            self.liability_share_value.into()
        ).ok_or(OrbitlenError::MathError)?;

        msg!(
            "deposit share value: {}\nliability share value: {}",
            asset_share_value,
            liability_share_value
        );

        self.asset_share_value = asset_share_value;
        self.liability_share_value = liability_share_value;

        Ok(())
    }

    pub fn get_asset_shares(&self, value: i128) -> Result<i128> {
        Ok(value.checked_div(self.asset_share_value.into()).ok_or(OrbitlenError::MathError)?)
    }

    pub fn change_asset_shares(&mut self, shares: i128) -> Result<()> {
        let total_asset_shares = self.total_asset_shares;
        self.total_asset_shares = total_asset_shares
            .checked_add(shares)
            .ok_or(OrbitlenError::MathError)?;
        Ok(())
    }

    pub fn get_liability_shares(&self, value: i128) -> Result<i128> {
        Ok(value.checked_div(self.liability_share_value.into()).ok_or(OrbitlenError::MathError)?)
    }

    pub fn change_liability_shares(&mut self, shares: i128) -> Result<()> {
        let total_liability_shares = self.total_liability_shares;
        self.total_liability_shares = total_liability_shares
            .checked_add(shares)
            .ok_or(OrbitlenError::MathError)?;

        Ok(())
    }
}

fn calc_interest_rate_accrual_state_changes(
    time_delta: u64,
    total_assets_amount: i128,
    total_liabilities_amount: i128,
    interest_rate_config: &InterestRateConfig,
    asset_share_value: i128,
    liability_share_value: i128
) -> Option<(i128, i128)> {
    let utilization_rate = total_liabilities_amount.checked_div(total_assets_amount)?;
    let (lending_apr, borrowing_apr) = interest_rate_config.calc_interest_rate(utilization_rate)?;

    msg!(
        "Accruing interest for {} seconds. Utilization rate: {}. Lending APR: {}. Borrowing APR: {}",
        time_delta,
        utilization_rate,
        lending_apr,
        borrowing_apr
    );

    Some((
        calc_accrued_interest_payment_per_period(lending_apr, time_delta, asset_share_value)?,
        calc_accrued_interest_payment_per_period(borrowing_apr, time_delta, liability_share_value)?,
    ))
}

fn calc_accrued_interest_payment_per_period(
    apr: i128,
    time_delta: u64,
    value: i128
) -> Option<i128> {
    let ir_per_period = apr.checked_mul(time_delta.into())?.checked_div(SECONDS_PER_YEAR)?;

    let new_value = value.checked_mul(1 + ir_per_period)?;

    Some(new_value)
}

#[zero_copy(unsafe)]
#[derive(PartialEq, Eq, Debug)]
pub struct BankConfig {
    pub asset_weight_init: i128,
    pub asset_weight_maint: i128,

    pub liability_weight_init: i128,
    pub liability_weight_maint: i128,

    pub interest_rate_config: InterestRateConfig,
    pub oracle_key: Pubkey,
}

impl Default for BankConfig {
    fn default() -> Self {
        Self {
            asset_weight_init: 0,
            asset_weight_maint: 0,
            liability_weight_init: 0,
            liability_weight_maint: 0,
            interest_rate_config: Default::default(),
            oracle_key: Pubkey::default(),
        }
    }
}

impl BankConfig {}

#[zero_copy(unsafe)]
#[derive(PartialEq, Eq, Default, Debug)]
pub struct InterestRateConfig {
    pub optimal_utilization_rate: i128,
    pub plateau_interest_rate: i128,
    pub max_interest_rate: i128,
}

impl InterestRateConfig {
    pub fn calc_interest_rate(&self, utilization_ratio: i128) -> Option<(i128, i128)> {
        let base_rate = self.interest_rate_curve(utilization_ratio)?;

        let lending_rate = base_rate.checked_mul(utilization_ratio)?;

        let borrowing_rate = base_rate;

        Some((lending_rate, borrowing_rate))
    }

    fn interest_rate_curve(&self, ur: i128) -> Option<i128> {
        let optimal_ur = self.optimal_utilization_rate;
        let plateau_ir = self.plateau_interest_rate;
        let max_ir = self.max_interest_rate;

        if ur <= optimal_ur {
            ur.checked_div(optimal_ur)?.checked_mul(plateau_ir)
        } else {
            (ur - optimal_ur)
                .checked_div(1 - optimal_ur)?
                .checked_mul(max_ir - plateau_ir)?
                .checked_add(plateau_ir)
        }
    }
}

#[derive(PartialEq, Eq, AnchorDeserialize, AnchorSerialize, Debug)]
pub struct BankConfigCompact {
    pub asset_weight_init: i128,
    pub asset_weight_maint: i128,
    pub liability_weight_init: i128,
    pub liability_weight_maint: i128,
    pub interest_rate_config: InterestRateConfigCompact,
    pub oracle_key: Pubkey,
}

impl From<BankConfigCompact> for BankConfig {
    fn from(config: BankConfigCompact) -> Self {
        Self {
            asset_weight_init: config.asset_weight_init,
            asset_weight_maint: config.asset_weight_maint,
            liability_weight_init: config.liability_weight_init,
            liability_weight_maint: config.liability_weight_maint,
            interest_rate_config: config.interest_rate_config.into(),
            oracle_key: config.oracle_key,
        }
    }
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize, PartialEq, Eq)]
pub struct InterestRateConfigCompact {
    pub optimal_utilization_rate: i128,
    pub plateau_interest_rate: i128,
    pub max_interest_rate: i128,
}

impl From<InterestRateConfigCompact> for InterestRateConfig {
    fn from(ir_config: InterestRateConfigCompact) -> Self {
        InterestRateConfig {
            optimal_utilization_rate: ir_config.optimal_utilization_rate,
            plateau_interest_rate: ir_config.plateau_interest_rate,
            max_interest_rate: ir_config.max_interest_rate
        }
    }
}

pub struct BankAccountWrapper<'a> {
    pub balance: &'a mut Balance,
    pub bank: &'a mut Bank,
}

impl<'a> BankAccountWrapper<'a> {
    pub fn find_or_create(
        bank_pk: &Pubkey,
        bank: &'a mut Bank,
        lending_account: &'a mut LendingAccount
    ) -> Result<BankAccountWrapper<'a>> {
        let balance_index = lending_account.balances
            .iter()
            .position(|balance| balance.bank_pk.eq(bank_pk));

        match balance_index {
            Some(balance_index) => {
                let balance = lending_account.balances
                    .get_mut(balance_index)
                    .ok_or_else(|| error!(OrbitlenError::BankAccountNotFound))?;

                Ok(Self { balance, bank })
            }
            None => {
                let empty_index = lending_account
                    .get_first_empty_balance()
                    .ok_or_else(|| error!(OrbitlenError::LendingAccountBalanceSlotsFull))?;

                lending_account.balances[empty_index] = Balance {
                    bank_pk: *bank_pk,
                    asset_shares: 0,
                    liability_shares: 0,
                    last_update: Clock::get()?.unix_timestamp as u64,
                };

                Ok(Self {
                    balance: lending_account.balances.get_mut(empty_index).unwrap(),
                    bank,
                })
            }
        }
    }

    pub fn deposit(&mut self, amount: i128) -> Result<()> {
        self.increase_balance_internal(amount)
    }

    fn increase_balance_internal(&mut self, balance_delta: i128) -> Result<()> {
        msg!("Balance increase: {} ", balance_delta);

        let balance = &mut self.balance;
        let bank = &mut self.bank;

        let current_liability_shares = balance.liability_shares.into();
        let current_liability_amount = bank.get_liability_amount(current_liability_shares)?;

        let (liability_amount_decrease, asset_amount_increase) = (
            min(current_liability_amount, balance_delta),
            max(
                balance_delta
                    .checked_sub(current_liability_amount)
                    .ok_or(OrbitlenError::MathError)?,
                0
            ),
        );

        let asset_shares_increase = bank.get_asset_shares(asset_amount_increase)?;
        balance.change_asset_shares(asset_shares_increase)?;
        bank.change_asset_shares(asset_shares_increase)?;

        let liability_shares_decrease = bank.get_liability_shares(liability_amount_decrease)?;
        balance.change_liability_shares(-liability_shares_decrease)?;
        bank.change_liability_shares(-liability_shares_decrease)?;

        Ok(())
    }

    pub fn borrow(&mut self, amount: i128) -> Result<()> {
        self.decrease_balance_internal(amount)
    }

    fn decrease_balance_internal(&mut self, balance_delta: i128) -> Result<()> {
        msg!("Balance decrease: {}", balance_delta);

        let balance = &mut self.balance;
        let bank = &mut self.bank;

        let current_asset_shares = balance.asset_shares;
        let current_asset_amount = bank.get_asset_amount(current_asset_shares)?;

        let (asset_amount_decrease, liability_amount_increase) = (
            min(current_asset_amount, balance_delta),
            max(
                balance_delta.checked_sub(current_asset_amount).ok_or(OrbitlenError::MathError)?,
                0
            ),
        );

        let asset_shares_decrease = bank.get_asset_shares(asset_amount_decrease)?;
        balance.change_asset_shares(-asset_shares_decrease)?;
        bank.change_asset_shares(-asset_shares_decrease)?;

        let liability_shares_increase = bank.get_liability_shares(liability_amount_increase)?;
        balance.change_liability_shares(liability_shares_increase)?;
        bank.change_liability_shares(liability_shares_increase)?;

        bank.check_utilization_ratio()?;

        Ok(())
    }

    pub fn withdraw_spl_transfer<'info>(
        &self,
        amount: u64,
        from: AccountInfo<'info>,
        to: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        mint: Option<&InterfaceAccount<'info, Mint>>,
        program: AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
        remaining_accounts: &[AccountInfo<'info>]
    ) -> Result<()> {
        self.bank.withdraw_spl_transfer(
            amount,
            from,
            to,
            authority,
            mint,
            program,
            signer_seeds,
            remaining_accounts
        )
    }

    pub fn deposit_spl_transfer<'info>(
        &self,
        amount: u64,
        from: AccountInfo<'info>,
        to: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        maybe_mint: Option<&InterfaceAccount<'info, Mint>>,
        program: AccountInfo<'info>,
        remaining_accounts: &[AccountInfo<'info>]
    ) -> Result<()> {
        self.bank.deposit_spl_transfer(
            amount,
            from,
            to,
            authority,
            maybe_mint,
            program,
            remaining_accounts
        )
    }
}

#[derive(Debug, Clone)]
pub enum BankVaultType {
    Liquidity,
    Insurance,
}

impl BankVaultType {
    pub fn get_seed(self) -> &'static [u8] {
        match self {
            BankVaultType::Liquidity => LIQUIDITY_VAULT_SEED.as_bytes(),
            BankVaultType::Insurance => INSURANCE_VAULT_SEED.as_bytes(),
        }
    }

    pub fn get_authority_seed(self) -> &'static [u8] {
        match self {
            BankVaultType::Liquidity => LIQUIDITY_VAULT_AUTHORITY_SEED.as_bytes(),
            BankVaultType::Insurance => INSURANCE_VAULT_AUTHORITY_SEED.as_bytes(),
        }
    }
}
