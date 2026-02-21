use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
    program::invoke,
    clock::Clock,
};
use spl_token::instruction as token_instruction;
use spl_associated_token_account::instruction as ata_instruction;

use crate::state::Vault;
use crate::events::{DepositEvent, WithdrawEvent, create_base_event};
use crate::{emit_event};

pub fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    bump: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let vault_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let authority = next_account_info(account_info_iter)?;
    let vault_token_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let associated_token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_sysvar = next_account_info(account_info_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let vault_size = std::mem::size_of::<Vault>() as u64;
    let rent = &Rent::from_account_info(rent_sysvar)?;
    let required_lamports = rent.minimum_balance(vault_size as usize);

    invoke(
        &system_instruction::create_account(
            authority.key,
            vault_account.key,
            required_lamports,
            vault_size,
            program_id,
        ),
        &[
            authority.clone(),
            vault_account.clone(),
            system_program.clone(), 
        ],
    )?;

    let vault = Vault {
        authority: *authority.key,
        bump,
        multi_sig: None,
        paused: false,
        emergency_admin: *authority.key,
        supported_tokens: vec![crate::state::TokenBalance {
            mint: *mint_account.key,
            balance: 0,
            yield_strategy: None,
        }],
        time_locks: vec![],
        proposals: vec![],
        next_proposal_id: 0,
        fee_config: crate::state::FeeConfig {
            deposit_fee_bps: 0,
            withdrawal_fee_bps: 0,
            fee_recipient: *authority.key,
        },
        total_value_locked: 0,
        total_fees_collected: 0,
        legacy_mint: Some(*mint_account.key),
        legacy_total_deposited: 0,
        governance_config: None,
        governance_proposals: vec![],
        next_governance_proposal_id: 0,
        vote_records: vec![],
        voter_registry: vec![],
    };

    vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

    msg!("Enhanced vault initialized successfully with legacy support");
    Ok(())
}