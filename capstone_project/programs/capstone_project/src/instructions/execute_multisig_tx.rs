use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke_signed;
use crate::state::Vault;
use crate::error::VaultError;
use crate::events::MultiSigTransactionExecutedEvent;

#[derive(Accounts)]
pub struct ExecuteMultiSigTx<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.authority.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,
    /// CHECK: Multisig signer PDA derived from vault key + nonce
    pub multisig_signer: UncheckedAccount<'info>,
    pub executor: Signer<'info>,
}

pub fn handler(ctx: Context<ExecuteMultiSigTx>, transaction_id: u64) -> Result<()> {
    let vault = &ctx.accounts.vault;

    // Check if multisig is initialized
    let multi_sig = vault.multi_sig.as_ref().ok_or(VaultError::MultisigNotInitialized)?;

    // Check if transaction exists
    require!(
        (transaction_id as usize) < vault.multi_sig_transactions.len(),
        VaultError::TransactionNotFound
    );

    let transaction = &vault.multi_sig_transactions[transaction_id as usize];

    // Check if already executed
    require!(!transaction.did_execute, VaultError::TransactionAlreadyExecuted);

    // Check if we have enough approvals
    let current_approvals = transaction.signers.iter().filter(|&&signed| signed).count();
    require!(
        current_approvals >= multi_sig.threshold as usize,
        VaultError::NotEnoughSigners
    );

    // Derive the multisig signer PDA
    let nonce = multi_sig.nonce;
    let vault_key = vault.key();
    let (expected_signer, bump) = Pubkey::find_program_address(
        &[vault_key.as_ref(), &[nonce]],
        ctx.program_id,
    );
    require!(
        expected_signer == ctx.accounts.multisig_signer.key(),
        VaultError::InvalidAccountData
    );

    // Build the instruction — mark both multisig_signer and vault as signers
    // since the vault PDA owns the assets and may need to sign transfers
    let vault_key_copy = vault_key;
    let ix = Instruction {
        program_id: transaction.program_id,
        accounts: transaction
            .accounts
            .iter()
            .map(|acc| {
                let is_pda_signer = acc.pubkey == ctx.accounts.multisig_signer.key()
                    || acc.pubkey == vault_key_copy;
                if is_pda_signer || acc.is_signer {
                    if acc.is_writable {
                        AccountMeta::new(acc.pubkey, true)
                    } else {
                        AccountMeta::new_readonly(acc.pubkey, true)
                    }
                } else if acc.is_writable {
                    AccountMeta::new(acc.pubkey, false)
                } else {
                    AccountMeta::new_readonly(acc.pubkey, false)
                }
            })
            .collect(),
        data: transaction.data.clone(),
    };

    // Execute with both vault PDA seeds and multisig signer PDA seeds
    let authority_key = vault.authority;
    let vault_bump = vault.bump;
    let vault_seeds: &[&[u8]] = &[b"vault", authority_key.as_ref(), &[vault_bump]];
    let ms_seeds: &[&[u8]] = &[vault_key.as_ref(), &[nonce], &[bump]];
    let signer_seeds = &[vault_seeds, ms_seeds];

    let remaining_accounts = ctx.remaining_accounts;
    invoke_signed(&ix, remaining_accounts, signer_seeds)?;

    // Mark transaction as executed
    let vault = &mut ctx.accounts.vault;
    vault.multi_sig_transactions[transaction_id as usize].did_execute = true;

    let clock = Clock::get()?;
    let target_program = vault.multi_sig_transactions[transaction_id as usize].program_id;

    emit!(MultiSigTransactionExecutedEvent {
        vault: vault.key(),
        transaction_id,
        executor: ctx.accounts.executor.key(),
        target_program,
        timestamp: clock.unix_timestamp,
    });

    msg!("Multi-sig transaction {} executed", transaction_id);
    Ok(())
}
