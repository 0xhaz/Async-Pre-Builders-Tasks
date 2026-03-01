use anchor_lang::prelude::*;
use crate::state::Vault;
use crate::error::VaultError;
use crate::events::MultiSigTransactionApprovedEvent;

#[derive(Accounts)]
pub struct ApproveMultiSigTx<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.authority.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,
    pub approver: Signer<'info>,
}

pub fn handler(ctx: Context<ApproveMultiSigTx>, transaction_id: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    // Check if multisig is initialized
    let multi_sig = vault.multi_sig.as_ref().ok_or(VaultError::MultisigNotInitialized)?;
    let threshold = multi_sig.threshold;

    // Find approver in owners list
    let owner_index = multi_sig
        .owners
        .iter()
        .position(|owner| *owner == ctx.accounts.approver.key())
        .ok_or(VaultError::InvalidOwner)?;

    // Check if transaction exists
    require!(
        (transaction_id as usize) < vault.multi_sig_transactions.len(),
        VaultError::TransactionNotFound
    );

    let transaction = &mut vault.multi_sig_transactions[transaction_id as usize];

    // Check if already executed
    require!(!transaction.did_execute, VaultError::TransactionAlreadyExecuted);

    // Check if already signed by this owner
    require!(!transaction.signers[owner_index], VaultError::TransactionAlreadySigned);

    // Approve
    transaction.signers[owner_index] = true;
    let current_approvals = transaction.signers.iter().filter(|&&signed| signed).count();

    let clock = Clock::get()?;
    emit!(MultiSigTransactionApprovedEvent {
        vault: vault.key(),
        transaction_id,
        approver: ctx.accounts.approver.key(),
        current_approvals: current_approvals as u32,
        required_approvals: threshold as u32,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Transaction {} approved ({} of {} approvals)",
        transaction_id,
        current_approvals,
        threshold
    );
    Ok(())
}
