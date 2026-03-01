use anchor_lang::prelude::*;
use crate::state::{MultiSigTransaction, TransactionAccount, Vault};
use crate::error::VaultError;
use crate::events::MultiSigTransactionCreatedEvent;

#[derive(Accounts)]
pub struct CreateMultiSigTx<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.authority.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub proposer: Signer<'info>,
}

pub fn handler(
    ctx: Context<CreateMultiSigTx>,
    target_program_id: Pubkey,
    accounts: Vec<TransactionAccount>,
    data: Vec<u8>,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    // Check if multisig is initialized
    let multi_sig = vault.multi_sig.as_ref().ok_or(VaultError::MultisigNotInitialized)?;

    // Check if proposer is an owner
    require!(
        multi_sig.owners.contains(&ctx.accounts.proposer.key()),
        VaultError::InvalidOwner
    );

    // Validate transaction data
    require!(!accounts.is_empty(), VaultError::InvalidTransactionData);

    let transaction_id = vault.multi_sig_transactions.len() as u64;

    // Find owner index and set initial approval
    let owner_index = multi_sig
        .owners
        .iter()
        .position(|owner| *owner == ctx.accounts.proposer.key())
        .ok_or(VaultError::InvalidOwner)?;

    let mut signers = vec![false; multi_sig.owners.len()];
    signers[owner_index] = true;

    let clock = Clock::get()?;
    let transaction = MultiSigTransaction {
        multisig: vault.key(),
        program_id: target_program_id,
        accounts: accounts.clone(),
        data,
        signers,
        did_execute: false,
        proposer: ctx.accounts.proposer.key(),
        created_at: clock.unix_timestamp,
    };

    vault.multi_sig_transactions.push(transaction);

    emit!(MultiSigTransactionCreatedEvent {
        vault: vault.key(),
        transaction_id,
        proposer: ctx.accounts.proposer.key(),
        target_program: target_program_id,
        instruction_count: accounts.len() as u32,
        timestamp: clock.unix_timestamp,
    });

    msg!("Multi-sig transaction {} created", transaction_id);
    Ok(())
}
