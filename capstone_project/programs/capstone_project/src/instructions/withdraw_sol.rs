use anchor_lang::prelude::*;
use crate::state::Vault;
use crate::error::VaultError;
use crate::events::SolWithdrawnEvent;

#[derive(Accounts)]
pub struct WithdrawSol<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.authority.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,
    /// CHECK: Recipient of SOL transfer
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WithdrawSol>, amount: u64) -> Result<()> {
    // Phase 1: Read state (temporary borrows)
    require!(!ctx.accounts.vault.paused, VaultError::VaultPaused);

    let vault_lamports = ctx.accounts.vault.to_account_info().lamports();
    require!(vault_lamports >= amount, VaultError::InvalidAmount);

    let withdrawal_fee_bps = ctx.accounts.vault.fee_config.withdrawal_fee_bps;
    let withdrawal_fee = if amount > 0 {
        (amount as u128 * withdrawal_fee_bps as u128 / 10000) as u64
    } else {
        0
    };
    let net_withdrawal_amount = amount - withdrawal_fee;

    // Phase 2: Direct lamport manipulation
    let vault_info = ctx.accounts.vault.to_account_info();
    **vault_info.try_borrow_mut_lamports()? -= net_withdrawal_amount;
    **ctx.accounts.recipient.try_borrow_mut_lamports()? += net_withdrawal_amount;

    // Phase 3: Update vault state
    let vault = &mut ctx.accounts.vault;
    vault.total_value_locked = vault.total_value_locked.saturating_sub(net_withdrawal_amount);
    vault.total_fees_collected += withdrawal_fee;

    let clock = Clock::get()?;
    emit!(SolWithdrawnEvent {
        vault: vault.key(),
        amount: net_withdrawal_amount,
        fee_amount: withdrawal_fee,
        recipient: ctx.accounts.recipient.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Withdrew {} SOL (fee: {})", net_withdrawal_amount, withdrawal_fee);
    Ok(())
}
