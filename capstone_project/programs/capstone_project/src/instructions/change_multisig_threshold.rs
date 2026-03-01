use anchor_lang::prelude::*;
use crate::state::Vault;
use crate::error::VaultError;
use crate::events::MultiSigThresholdUpdatedEvent;

#[derive(Accounts)]
pub struct ChangeMultiSigThreshold<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.authority.as_ref()],
        bump = vault.bump,
        has_one = authority @ VaultError::InsufficientAuthority,
    )]
    pub vault: Account<'info, Vault>,
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<ChangeMultiSigThreshold>, threshold: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    let multi_sig = vault.multi_sig.as_mut().ok_or(VaultError::MultisigNotInitialized)?;

    // Validate threshold
    require!(
        threshold > 0 && threshold <= multi_sig.owners.len() as u64,
        VaultError::InvalidThreshold
    );

    let old_threshold = multi_sig.threshold;
    multi_sig.threshold = threshold;

    let clock = Clock::get()?;
    emit!(MultiSigThresholdUpdatedEvent {
        vault: vault.key(),
        authority: ctx.accounts.authority.key(),
        old_threshold,
        new_threshold: threshold,
        timestamp: clock.unix_timestamp,
    });

    msg!("Multi-sig threshold changed from {} to {}", old_threshold, threshold);
    Ok(())
}
