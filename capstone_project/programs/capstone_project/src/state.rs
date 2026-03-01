use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum VoteType {
    For,
    Against,
    Abstain,
}

pub type GovernanceInstruction = Vec<u8>;

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct FeeConfig {
    pub deposit_fee_bps: u16,
    pub withdrawal_fee_bps: u16,
    pub fee_recipient: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct SupportedToken {
    pub mint: Pubkey,
    pub bump: u8,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub is_active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct TokenBalance {
    pub mint: Pubkey,
    pub balance: u64,
    pub last_updated: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct TimeLock {
    pub beneficiary: Pubkey,
    pub amount: u64,
    pub start_time: i64,
    pub duration: i64,
    pub cliff_duration: Option<i64>,
    pub is_linear: bool,
    pub claimed_amount: u64,
    pub end_time: i64,
    pub cliff_time: i64,
    pub released_amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct Proposal {
    pub id: u64,
    pub instruction_data: Vec<u8>,
    pub approvals: Vec<Pubkey>,
    pub executed: bool,
    pub created_at: i64,
    pub proposer: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct GovernanceProposal {
    pub id: u64,
    pub proposer: Pubkey,
    pub title: String,
    pub description: String,
    pub instructions: Vec<Vec<u8>>,
    pub for_votes: u64,
    pub against_votes: u64,
    pub abstain_votes: u64,
    pub created_at: i64,
    pub end_time: i64,
    pub executed: bool,
    pub queued: bool,
    pub eta: Option<i64>,
    pub start_time: i64,
    pub cancelled: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct VoteRecord {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub vote_type: VoteType,
    pub voting_power: u64,
    pub voted_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct VoterRegistry {
    pub voter: Pubkey,
    pub voting_power: u64,
    pub registered_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct GovernanceConfig {
    pub voting_token_mint: Pubkey,
    pub quorum_threshold: u16,
    pub proposal_threshold: u64,
    pub voting_period: i64,
    pub time_lock_delay: i64,
    pub execution_threshold: u16,
    pub timelock_delay: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct MultiSig {
    pub owners: Vec<Pubkey>,
    pub threshold: u64,
    pub nonce: u8,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct MultiSigTransaction {
    pub multisig: Pubkey,
    pub program_id: Pubkey,
    pub accounts: Vec<TransactionAccount>,
    pub data: Vec<u8>,
    pub signers: Vec<bool>,
    pub did_execute: bool,
    pub proposer: Pubkey,
    pub created_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TransactionAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct YieldStrategyConfig {
    pub token_mint: Pubkey,
    pub strategy_program: Pubkey,
    pub auto_compound: bool,
    pub last_harvested_slot: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct EmergencyActionLog {
    pub timestamp: i64,
    pub admin: Pubkey,
    pub action: u8,
    pub details: Vec<u8>,
}

#[account]
#[derive(Debug)]
pub struct Vault {
    pub authority: Pubkey,
    pub bump: u8,
    pub emergency_admin: Pubkey,
    pub paused: bool,
    pub supported_tokens: Vec<SupportedToken>,
    pub token_balances: Vec<TokenBalance>,
    pub time_locks: Vec<TimeLock>,
    pub proposals: Vec<Proposal>,
    pub next_proposal_id: u64,
    pub fee_config: FeeConfig,
    pub total_value_locked: u64,
    pub total_fees_collected: u64,
    pub legacy_mint: Option<Pubkey>,
    pub legacy_total_deposited: u64,
    pub governance_config: Option<GovernanceConfig>,
    pub governance_proposals: Vec<GovernanceProposal>,
    pub next_governance_proposal_id: u64,
    pub vote_records: Vec<VoteRecord>,
    pub voter_registry: Vec<VoterRegistry>,
    pub multi_sig: Option<MultiSig>,
    pub multi_sig_transactions: Vec<MultiSigTransaction>,
    pub yield_strategies: Vec<YieldStrategyConfig>,
    pub emergency_logs: Vec<EmergencyActionLog>,
}

impl Vault {
    pub const INIT_SPACE: usize = 10232;
}
