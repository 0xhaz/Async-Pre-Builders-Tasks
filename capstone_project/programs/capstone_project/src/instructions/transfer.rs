use anchor_lang::prelude::*;
use crate::state::Vault;
use crate::error::VaultError;
use crate::events::SolTransferredEvent;

#[derive(Accounts)]
pub struct TransferSol<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.authority.as_ref()],
        bump = vault.bump,
        has_one = authority @ VaultError::InsufficientAuthority,
    )]
    pub vault: Account<'info, Vault>,
    /// CHECK: Recipient of SOL transfer
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<TransferSol>, amount: u64) -> Result<()> {
    // Phase 1: Read state (temporary borrows)
    require!(!ctx.accounts.vault.paused, VaultError::VaultPaused);

    let vault_lamports = ctx.accounts.vault.to_account_info().lamports();
    require!(vault_lamports >= amount, VaultError::InvalidAmount);

    let withdrawal_fee_bps = ctx.accounts.vault.fee_config.withdrawal_fee_bps;
    let transfer_fee = if amount > 0 {
        (amount as u128 * withdrawal_fee_bps as u128 / 10000) as u64
    } else {
        0
    };
    let net_transfer_amount = amount - transfer_fee;

    // Phase 2: Direct lamport manipulation
    let vault_info = ctx.accounts.vault.to_account_info();
    **vault_info.try_borrow_mut_lamports()? -= net_transfer_amount;
    **ctx.accounts.recipient.try_borrow_mut_lamports()? += net_transfer_amount;

    // Phase 3: Update vault state
    let vault = &mut ctx.accounts.vault;
    vault.total_value_locked = vault.total_value_locked.saturating_sub(net_transfer_amount);
    vault.total_fees_collected += transfer_fee;

    let clock = Clock::get()?;
    emit!(SolTransferredEvent {
        vault: vault.key(),
        authority: ctx.accounts.authority.key(),
        amount: net_transfer_amount,
        fee_amount: transfer_fee,
        recipient: ctx.accounts.recipient.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Transferred {} SOL (fee: {})", net_transfer_amount, transfer_fee);
    Ok(())
}
