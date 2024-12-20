use crate::{ error::OrbitlenError, state::* };
use anchor_lang::prelude::*;
use anchor_spl::token_interface::*;

pub fn maybe_take_bank_mint<'info>(
    remaining_accounts: &mut &'info [AccountInfo<'info>],
    bank: &Bank,
    token_program: &Pubkey
) -> Result<InterfaceAccount<'info, Mint>> {
    match *token_program {
        anchor_spl::token::ID | anchor_spl::token_2022::ID => {
            let (maybe_mint, remaining) = remaining_accounts
                .split_first()
                .ok_or(OrbitlenError::MintRequired)?;

            msg!("maybe_take_bank_mint: maybe_mint: {:?}", maybe_mint.key);

            *remaining_accounts = remaining;

            if bank.mint != *maybe_mint.key {
                return err!(OrbitlenError::MintRequired);
            }

            InterfaceAccount::try_from(maybe_mint)
        }

        _ => panic!("unsupported token program"),
    }
}
