use anchor_lang::prelude::*;

pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;
use state::TransactionAccount;

declare_id!("GJmkK7megeRLsTd1XuPGqXZh7ZFusAzKqdUqNx5Gqyb9");

#[program]
pub mod capstone_project {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, amount)
    }

    pub fn withdraw_sol(ctx: Context<WithdrawSol>, amount: u64) -> Result<()> {
        instructions::withdraw_sol::handler(ctx, amount)
    }

    pub fn transfer_sol(ctx: Context<TransferSol>, amount: u64) -> Result<()> {
        instructions::transfer::handler(ctx, amount)
    }

    pub fn initialize_multi_sig(
        ctx: Context<InitializeMultiSig>,
        owners: Vec<Pubkey>,
        threshold: u64,
        nonce: u8,
    ) -> Result<()> {
        instructions::initialize_multisig::handler(ctx, owners, threshold, nonce)
    }

    pub fn add_supported_token(ctx: Context<AddSupportedToken>) -> Result<()> {
        instructions::add_supported_token::handler(ctx)
    }

    pub fn create_multisig_transaction(
        ctx: Context<CreateMultiSigTx>,
        target_program_id: Pubkey,
        accounts: Vec<TransactionAccount>,
        data: Vec<u8>,
    ) -> Result<()> {
        instructions::create_multisig_tx::handler(ctx, target_program_id, accounts, data)
    }

    pub fn approve_multisig_transaction(
        ctx: Context<ApproveMultiSigTx>,
        transaction_id: u64,
    ) -> Result<()> {
        instructions::approve_multisig_tx::handler(ctx, transaction_id)
    }

    pub fn execute_multisig_transaction(
        ctx: Context<ExecuteMultiSigTx>,
        transaction_id: u64,
    ) -> Result<()> {
        instructions::execute_multisig_tx::handler(ctx, transaction_id)
    }

    pub fn set_multisig_owners(
        ctx: Context<SetMultiSigOwners>,
        owners: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::set_multisig_owners::handler(ctx, owners)
    }

    pub fn change_multisig_threshold(
        ctx: Context<ChangeMultiSigThreshold>,
        threshold: u64,
    ) -> Result<()> {
        instructions::change_multisig_threshold::handler(ctx, threshold)
    }
}
