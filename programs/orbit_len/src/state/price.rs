use anchor_lang::prelude::*;
use switchboard_on_demand::on_demand::accounts::pull_feed::PullFeedAccountData;
use rust_decimal::Decimal;

use crate::error::OrbitlenError;

use super::BankConfig;

pub fn fetch_feed_price<'info>(feed: &AccountInfo<'_>, bank_config: &BankConfig) -> Result<f64> {
    require_keys_eq!(*feed.key, bank_config.oracle_key, OrbitlenError::InvalidPriceFeedPk);
    let feed_account = feed.data.borrow();

    // Docs at: https://switchboard-on-demand-rust-docs.web.app/on_demand/accounts/pull_feed/struct.PullFeedAccountData.html
    let feed = PullFeedAccountData::parse(feed_account).unwrap();

    // Get the value,

    let value = feed.value().unwrap_or(Decimal::ZERO);
    println!("The {} value is: {:?}", bank_config.oracle_key, value);
    value.try_into().map_err(|_| OrbitlenError::FetchPriceFailed.into())
}

pub fn calc_amount(value: u64, price: f64, mint_decimals: u8) -> Result<u64> {
    let qt = value
        .checked_mul(mint_decimals as u64)
        .ok_or(OrbitlenError::MathError)?
        .checked_div(price as u64)
        .ok_or(OrbitlenError::MathError)?;

    Ok(qt)
}

pub fn calc_value(amount: u64, price: f64, mint_decimals: u8) -> Result<u64> {
    msg!("amount: {}, price: {}", amount, price);

    let value = amount
        .checked_mul(price as u64)
        .ok_or(OrbitlenError::MathError)?
        .checked_div(mint_decimals as u64)
        .ok_or(OrbitlenError::MathError)?;

    Ok(value)
}
