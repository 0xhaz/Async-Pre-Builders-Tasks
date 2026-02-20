use crate::state::Vault;
use crate::VaultError;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Helper function to get vault from account
pub fn get_vault_from_account(account: &AccountInfo) -> Result<Vault, ProgramError> {
    let account_data = account.data.borrow();
    Vault::try_from_slice(&account_data).map_err(|_| ProgramError::InvalidAccountData)
}

// Helper function to save vault to account
pub fn save_vault_to_account(vault: &Vault, account: &AccountInfo) -> Result<(), ProgramError> {
    let mut account_data = account.data.borrow_mut();
    vault.serialize(&mut account_data.as_mut()).map_err(|_| ProgramError::InvalidAccountData)
}

// Helper function to validate account ownership
pub fn validate_account_ownership(account: &AccountInfo, expected_owner: &Pubkey) -> Result<(), ProgramError> {
    if account.owner != expected_owner {
        return Err(ProgramError::InvalidAccountOwner);

    }
    Ok(())
}