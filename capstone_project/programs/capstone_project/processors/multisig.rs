use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    clock::Clock,
    sysvar::Sysvar,
};

use crate::instruction::VaultInstruction;
use crate::state::{Vault, MultiSigAuthority, Proposal};
use crate::events::{create_base_event, MultiSigInitializedEvent};
use crate::emit_event;

pub fn process_initialize_multi_sig(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    authorities: Vec<Pubkey>,
    threshold: u8,
    _bump: u8
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let vault_account = next_account_info(account_info_iter)?;
    let initializer = next_account_info(account_info_iter)?;
    let clock_sysvar = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if vault_account.owner != _program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let vault_data = vault_account.data.borrow();
    let mut vault = Vault::try_from_slice(&vault_data)?;

    if vault.authority != *initializer.key {
        return Err(ProgramError::InvalidAccountData);
    }

    vault.multi_sig = Some(MultiSigAuthority {
        authorities: authorities.clone(),
        threshold,
        nonce: 0,
    });

    drop(vault_data);
    vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

    let clock = Clock::from_account_info(clock_sysvar)?;
    let multi_sig_event = MultiSigInitializedEvent {
        base: create_base_event(*vault_account.key, *initializer.key, "multi_sig_initialized", &clock),
        authorities,
        threshold,
    };
    emit_event(&multi_sig_event)?;

    msg!("Multi-signature initialized with {} authorities and threshold {}", vault.multi_sig.as_ref().unwrap().authorities.len(), threshold);
    Ok(())
}