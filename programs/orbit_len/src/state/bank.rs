use anchor_lang::prelude::*;
use crate::{ state::account::*, error::*, constants::* };
use std::{ cmp::{ max, min }, fmt::Debug };
use anchor_spl::token_interface::*;

#[account(zero_copy(unsafe))]
#[derive(Debug, PartialEq, Default, InitSpace)]
pub struct Bank {
    pub mint: Pubkey,
    pub mint_decimals: u8,

    pub asset_share_value: u64,

    pub liability_share_value: u64,

    pub liquidity_vault: Pubkey,
    pub liquidity_vault_bump: u8,
    pub liquidity_vault_authority_bump: u8,

    pub total_liability_shares: u64,
    pub total_asset_shares: u64,

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
        liquidity_vault_bump: u8,
        liquidity_vault_authority_bump: u8
    ) -> Bank {
        Bank {
            mint,
            mint_decimals,
            asset_share_value: 1,
            liability_share_value: 1,
            liquidity_vault,
            liquidity_vault_bump,
            liquidity_vault_authority_bump,
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

    pub fn get_liability_amount(&self, shares: u64) -> Result<u64> {
        shares.checked_mul(self.liability_share_value.into()).ok_or(OrbitlenError::MathError.into())
    }

    pub fn get_asset_amount(&self, shares: u64) -> Result<u64> {
        shares.checked_mul(self.asset_share_value).ok_or(OrbitlenError::MathError.into())
    }

    pub fn accrue_interest(&mut self, current_timestamp: i64) -> Result<()> {
        msg!("===accrue_interest===");
        let time_delta: u64 = (current_timestamp - self.last_update).try_into().unwrap();
        msg!("time_delta: {}", time_delta);

        if time_delta == 0 {
            return Ok(());
        }

        let total_assets = self.get_asset_amount(self.total_asset_shares.into())?;
        let total_liabilities = self.get_liability_amount(self.total_liability_shares.into())?;

        self.last_update = current_timestamp;

        msg!("total_assets: {}", total_assets);
        msg!("total_liabilities: {}", total_liabilities);
        if total_assets == 0 || total_liabilities == 0 {
            return Ok(());
        }

        let debug_self_asset_share_value = self.asset_share_value;
        let debug_self_liability_share_value = self.liability_share_value;

        msg!("asset_share_value: {}", debug_self_asset_share_value);
        msg!("liability_share_value: {}", debug_self_liability_share_value);
        msg!("interest_rate_config: {:?}", self.config.interest_rate_config);

        let (asset_share_value, liability_share_value) = calc_interest_rate_accrual_state_changes(
            time_delta,
            total_assets,
            total_liabilities,
            &self.config.interest_rate_config,
            self.asset_share_value.into(),
            self.liability_share_value.into()
        ).ok_or_else(|| {
            let debug_self_asset_share_value = self.asset_share_value;
            let debug_self_liability_share_value = self.liability_share_value;
            msg!(
                "Failed to calculate interest: time_delta={}, total_assets={}, total_liabilities={}, asset_share_value={}, liability_share_value={}",
                time_delta,
                total_assets,
                total_liabilities,
                debug_self_asset_share_value,
                debug_self_liability_share_value
            );
            OrbitlenError::MathError
        })?;

        msg!(
            "deposit share value: {}\nliability share value: {}",
            asset_share_value,
            liability_share_value
        );

        self.asset_share_value = asset_share_value;
        self.liability_share_value = liability_share_value;

        Ok(())
    }

    pub fn get_asset_shares(&self, value: u64) -> Result<u64> {
        Ok(value.checked_div(self.asset_share_value.into()).ok_or(OrbitlenError::MathError)?)
    }

    pub fn change_asset_shares(&mut self, shares: i64) -> Result<()> {
        let total_asset_shares = self.total_asset_shares as i64;
        self.total_asset_shares = (total_asset_shares + shares) as u64;
        Ok(())
    }

    pub fn get_liability_shares(&self, value: u64) -> Result<u64> {
        Ok(value.checked_div(self.liability_share_value.into()).ok_or(OrbitlenError::MathError)?)
    }

    pub fn change_liability_shares(&mut self, shares: i64) -> Result<()> {
        let total_liability_shares = self.total_liability_shares as i64;
        self.total_liability_shares = (total_liability_shares + shares) as u64;
        Ok(())
    }
}

fn calc_interest_rate_accrual_state_changes(
    time_delta: u64,
    total_assets_amount: u64,
    total_liabilities_amount: u64,
    interest_rate_config: &InterestRateConfig,
    asset_share_value: u64,
    liability_share_value: u64
) -> Option<(u64, u64)> {
    msg!("=== calc_interest_rate_accrual_state_changes ===");
    let utilization_rate = (total_liabilities_amount as f64) / (total_assets_amount as f64);
    msg!("utilization_rate: {}", utilization_rate);

    let (lending_apr, borrowing_apr) = interest_rate_config.calc_interest_rate(
        utilization_rate as f32
    )?;
    msg!("lending_apr: {}, borrowing_apr: {}", lending_apr, borrowing_apr);
    Some((
        calc_accrued_interest_payment_per_period(
            lending_apr,
            time_delta as f32,
            asset_share_value as f32
        )?,
        calc_accrued_interest_payment_per_period(
            borrowing_apr,
            time_delta as f32,
            liability_share_value as f32
        )?,
    ))
}

fn calc_accrued_interest_payment_per_period(apr: f32, time_delta: f32, value: f32) -> Option<u64> {
    msg!("APR: {}, time_delta: {}, value: {}", apr, time_delta, value);
    let ir_per_period = (apr * time_delta) / (SECONDS_PER_YEAR as f32);
    msg!("Interest rate per period: {}", ir_per_period);
    let new_value = value * (1.0 + ir_per_period);
    msg!("New value: {}", new_value);
    Some(new_value as u64)
}

#[zero_copy(unsafe)]
#[derive(PartialEq, Debug, InitSpace, Default)]
pub struct BankConfig {
    pub interest_rate_config: InterestRateConfig,
    pub feed_data_key: Pubkey,
}

impl BankConfig {}

#[zero_copy(unsafe)]
#[derive(PartialEq, Default, Debug, InitSpace)]
pub struct InterestRateConfig {
    pub optimal_utilization_rate: u16,
    pub plateau_interest_rate: u16,
    pub max_interest_rate: u16,
}

impl InterestRateConfig {
    pub fn as_float(&self, value: u16) -> f32 {
        (value as f32) / 100.0
    }
    pub fn calc_interest_rate(&self, utilization_ratio: f32) -> Option<(f32, f32)> {
        msg!("=== Interest Rate Calculation ===");
        msg!("utilization_ratio: {}", utilization_ratio);
        let base_rate = self.interest_rate_curve(utilization_ratio)?;
        let lending_rate = base_rate * utilization_ratio;
        let borrowing_rate = base_rate;
        Some((lending_rate, borrowing_rate))
    }

    fn interest_rate_curve(&self, ur: f32) -> Option<f32> {
        let optimal_ur = self.as_float(self.optimal_utilization_rate);
        let plateau_ir = self.as_float(self.plateau_interest_rate);
        let max_ir = self.as_float(self.max_interest_rate);
        if ur <= optimal_ur {
            Some((ur / optimal_ur) * plateau_ir)
        } else {
            Some(((ur - optimal_ur) / (1.0 - optimal_ur)) * (max_ir - plateau_ir) + plateau_ir)
        }
    }
}

#[derive(PartialEq, AnchorDeserialize, AnchorSerialize, Debug)]
pub struct BankConfigCompact {
    pub interest_rate_config: InterestRateConfigCompact,
    pub feed_data_key: Pubkey,
}

impl From<BankConfigCompact> for BankConfig {
    fn from(config: BankConfigCompact) -> Self {
        Self {
            interest_rate_config: config.interest_rate_config.into(),
            feed_data_key: config.feed_data_key,
        }
    }
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize, PartialEq)]
pub struct InterestRateConfigCompact {
    pub optimal_utilization_rate: u16,
    pub plateau_interest_rate: u16,
    pub max_interest_rate: u16,
}

impl From<InterestRateConfigCompact> for InterestRateConfig {
    fn from(ir_config: InterestRateConfigCompact) -> Self {
        InterestRateConfig {
            optimal_utilization_rate: ir_config.optimal_utilization_rate,
            plateau_interest_rate: ir_config.plateau_interest_rate,
            max_interest_rate: ir_config.max_interest_rate,
        }
    }
}

pub struct BankAccountWrapper<'a> {
    pub balance: &'a mut Balance,
    pub bank: &'a mut Bank,
}

impl<'a> BankAccountWrapper<'a> {
    pub fn find(
        bank_pk: &Pubkey,
        bank: &'a mut Bank,
        lending_account: &'a mut LendingAccount
    ) -> Result<BankAccountWrapper<'a>> {
        let balance = lending_account.balances
            .iter_mut()
            .find(|balance| balance.bank_pk.eq(bank_pk))
            .ok_or_else(|| error!(OrbitlenError::BankAccountNotFound))?;

        Ok(Self { balance, bank })
    }
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

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        self.decrease_balance_internal(amount as i64)
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        self.increase_balance_internal(amount as i64)
    }

    fn increase_balance_internal(&mut self, balance_delta: i64) -> Result<()> {
        msg!("Balance increase: {} ", balance_delta);

        let balance = &mut self.balance;
        let bank = &mut self.bank;

        let current_liability_shares = balance.liability_shares.into();
        let current_liability_amount = bank.get_liability_amount(current_liability_shares)? as i64;

        let (liability_amount_decrease, asset_amount_increase) = (
            min(current_liability_amount, balance_delta),
            max(
                balance_delta
                    .checked_sub(current_liability_amount)
                    .ok_or(OrbitlenError::MathError)?,
                0
            ),
        );

        let asset_shares_increase = bank.get_asset_shares(asset_amount_increase as u64)?;
        balance.change_asset_shares(asset_shares_increase as i64)?;
        bank.change_asset_shares(asset_shares_increase as i64)?;

        let liability_shares_decrease = bank.get_liability_shares(
            liability_amount_decrease as u64
        )?;
        balance.change_liability_shares(-(liability_shares_decrease as i64))?;
        bank.change_liability_shares(-(liability_shares_decrease as i64))?;

        Ok(())
    }

    pub fn decrease_balance_in_liquidation(&mut self, amount: u64) -> Result<()> {
        self.decrease_balance_internal(amount as i64)
    }

    pub fn increase_balance(&mut self, amount: u64) -> Result<()> {
        self.increase_balance_internal(amount as i64)
    }

    pub fn borrow(&mut self, amount: u64) -> Result<()> {
        self.decrease_balance_internal(amount as i64)
    }

    pub fn increase_balance_in_liquidation(&mut self, amount: u64) -> Result<()> {
        self.increase_balance_internal(amount as i64)
    }

    fn decrease_balance_internal(&mut self, balance_delta: i64) -> Result<()> {
        msg!("Balance decrease: {}", balance_delta);

        let balance = &mut self.balance;
        let bank = &mut self.bank;

        let current_asset_shares = balance.asset_shares;
        let current_asset_amount = bank.get_asset_amount(current_asset_shares)? as i64;

        msg!("current_asset_amount: {}, balance_delta: {}", current_asset_amount, balance_delta);

        let (asset_amount_decrease, liability_amount_increase) = (
            min(current_asset_amount, balance_delta),
            max(
                balance_delta.checked_sub(current_asset_amount).ok_or(OrbitlenError::MathError)?,
                0
            ),
        );

        let asset_shares_decrease = bank.get_asset_shares(asset_amount_decrease as u64)?;
        balance.change_asset_shares(-(asset_shares_decrease as i64))?;
        bank.change_asset_shares(-(asset_shares_decrease as i64))?;

        let liability_shares_increase = bank.get_liability_shares(
            liability_amount_increase as u64
        )?;
        balance.change_liability_shares(liability_shares_increase as i64)?;
        bank.change_liability_shares(liability_shares_increase as i64)?;

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
