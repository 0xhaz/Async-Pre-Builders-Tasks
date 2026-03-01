use anchor_lang::prelude::*;
use crate::state::{FeeConfig, Vault};
use crate::events::VaultInitializedEvent;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Vault::INIT_SPACE,
        seeds = [b"vault", authority.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Stored as the emergency admin pubkey
    pub emergency_admin: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.authority = ctx.accounts.authority.key();
    vault.bump = ctx.bumps.vault;
    vault.emergency_admin = ctx.accounts.emergency_admin.key();
    vault.paused = false;
    vault.fee_config = FeeConfig {
        deposit_fee_bps: 0,
        withdrawal_fee_bps: 0,
        fee_recipient: ctx.accounts.authority.key(),
    };

    let clock = Clock::get()?;
    emit!(VaultInitializedEvent {
        vault: vault.key(),
        authority: ctx.accounts.authority.key(),
        emergency_admin: ctx.accounts.emergency_admin.key(),
        bump: vault.bump,
        timestamp: clock.unix_timestamp,
    });

    msg!("Vault initialized successfully with PDA: {}", vault.key());
    Ok(())
}
