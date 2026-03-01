use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("The given owner is not part of this multisig")]
    InvalidOwner,
    #[msg("Not enough owners signed this transaction")]
    NotEnoughSigners,
    #[msg("The given transaction has already been executed")]
    TransactionAlreadyExecuted,
    #[msg("The owner has already signed this transaction")]
    TransactionAlreadySigned,
    #[msg("Threshold must be less than or equal to the number of owners")]
    InvalidThreshold,
    #[msg("Multisig has not been initialized for this vault")]
    MultisigNotInitialized,
    #[msg("Transaction with the given ID was not found")]
    TransactionNotFound,
    #[msg("Insufficient authority to perform this operation")]
    InsufficientAuthority,
    #[msg("Invalid transaction data provided")]
    InvalidTransactionData,
    #[msg("Unauthorized access to this operation")]
    UnauthorizedAccess,
    #[msg("Invalid account data")]
    InvalidAccountData,
    #[msg("Account is not rent exempt")]
    AccountNotRentExempt,
    #[msg("Invalid account owner")]
    InvalidAccountOwner,
    #[msg("Arithmetic operation overflow")]
    ArithmeticOverflow,
    #[msg("Invalid amount specified")]
    InvalidAmount,
    #[msg("Vault is paused")]
    VaultPaused,
    #[msg("Token is not supported")]
    TokenNotSupported,
    #[msg("Token is already supported")]
    TokenAlreadySupported,
    #[msg("Duplicate owners detected")]
    DuplicateOwners,
}
