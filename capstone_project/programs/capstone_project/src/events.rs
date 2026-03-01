use anchor_lang::prelude::*;

#[event]
pub struct VaultInitializedEvent {
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub emergency_admin: Pubkey,
    pub bump: u8,
    pub timestamp: i64,
}

#[event]
pub struct TokenDepositedEvent {
    pub vault: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub fee_amount: u64,
    pub depositor: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct TokenWithdrawnEvent {
    pub vault: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub fee_amount: u64,
    pub recipient: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct SolWithdrawnEvent {
    pub vault: Pubkey,
    pub amount: u64,
    pub fee_amount: u64,
    pub recipient: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct SolTransferredEvent {
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub amount: u64,
    pub fee_amount: u64,
    pub recipient: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MultiSigInitializedEvent {
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub owners: Vec<Pubkey>,
    pub threshold: u64,
    pub nonce: u8,
    pub timestamp: i64,
}

#[event]
pub struct TokenAddedEvent {
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub vault_token_account: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MultiSigTransactionCreatedEvent {
    pub vault: Pubkey,
    pub transaction_id: u64,
    pub proposer: Pubkey,
    pub target_program: Pubkey,
    pub instruction_count: u32,
    pub timestamp: i64,
}

#[event]
pub struct MultiSigTransactionApprovedEvent {
    pub vault: Pubkey,
    pub transaction_id: u64,
    pub approver: Pubkey,
    pub current_approvals: u32,
    pub required_approvals: u32,
    pub timestamp: i64,
}

#[event]
pub struct MultiSigTransactionExecutedEvent {
    pub vault: Pubkey,
    pub transaction_id: u64,
    pub executor: Pubkey,
    pub target_program: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MultiSigOwnersUpdatedEvent {
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub old_owners: Vec<Pubkey>,
    pub new_owners: Vec<Pubkey>,
    pub timestamp: i64,
}

#[event]
pub struct MultiSigThresholdUpdatedEvent {
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub old_threshold: u64,
    pub new_threshold: u64,
    pub timestamp: i64,
}
