use anchor_lang::prelude::*;
use crate::state::Vault;
use crate::error::VaultError;
use crate::events::MultiSigOwnersUpdatedEvent;

#[derive(Accounts)]
pub struct SetMultiSigOwners<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.authority.as_ref()],
        bump = vault.bump,
        has_one = authority @ VaultError::InsufficientAuthority,
    )]
    pub vault: Account<'info, Vault>,
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<SetMultiSigOwners>, owners: Vec<Pubkey>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    let multi_sig = vault.multi_sig.as_mut().ok_or(VaultError::MultisigNotInitialized)?;

    // Validate no duplicate owners
    let mut unique_owners = owners.clone();
    unique_owners.sort();
    unique_owners.dedup();
    require!(unique_owners.len() == owners.len(), VaultError::DuplicateOwners);

    let old_owners = multi_sig.owners.clone();

    // Adjust threshold if necessary
    if (owners.len() as u64) < multi_sig.threshold {
        multi_sig.threshold = owners.len() as u64;
    }

    multi_sig.owners = owners.clone();

    let clock = Clock::get()?;
    emit!(MultiSigOwnersUpdatedEvent {
        vault: vault.key(),
        authority: ctx.accounts.authority.key(),
        old_owners,
        new_owners: owners,
        timestamp: clock.unix_timestamp,
    });

    msg!("Multi-sig owners updated");
    Ok(())
}
