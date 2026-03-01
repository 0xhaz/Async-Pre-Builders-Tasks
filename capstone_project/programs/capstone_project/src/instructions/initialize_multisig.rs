use anchor_lang::prelude::*;
use crate::state::{MultiSig, Vault};
use crate::error::VaultError;
use crate::events::MultiSigInitializedEvent;

#[derive(Accounts)]
pub struct InitializeMultiSig<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.authority.as_ref()],
        bump = vault.bump,
        has_one = authority @ VaultError::InsufficientAuthority,
    )]
    pub vault: Account<'info, Vault>,
    pub authority: Signer<'info>,
}

pub fn handler(
    ctx: Context<InitializeMultiSig>,
    owners: Vec<Pubkey>,
    threshold: u64,
    nonce: u8,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    // Validate threshold
    require!(
        threshold > 0 && threshold <= owners.len() as u64,
        VaultError::InvalidThreshold
    );

    // Validate no duplicate owners
    let mut unique_owners = owners.clone();
    unique_owners.sort();
    unique_owners.dedup();
    require!(unique_owners.len() == owners.len(), VaultError::DuplicateOwners);

    vault.multi_sig = Some(MultiSig {
        owners: owners.clone(),
        threshold,
        nonce,
        bump: 0,
    });

    let clock = Clock::get()?;
    emit!(MultiSigInitializedEvent {
        vault: vault.key(),
        authority: ctx.accounts.authority.key(),
        owners,
        threshold,
        nonce,
        timestamp: clock.unix_timestamp,
    });

    msg!("Multi-signature initialized with {} owners and threshold {}", unique_owners.len(), threshold);
    Ok(())
}
