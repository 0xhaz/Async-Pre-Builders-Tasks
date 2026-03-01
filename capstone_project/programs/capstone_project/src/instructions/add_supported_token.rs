use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{SupportedToken, Vault};
use crate::error::VaultError;
use crate::events::TokenAddedEvent;

#[derive(Accounts)]
pub struct AddSupportedToken<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.authority.as_ref()],
        bump = vault.bump,
        has_one = authority @ VaultError::InsufficientAuthority,
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = vault,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddSupportedToken>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let mint = ctx.accounts.token_mint.key();

    // Check if token is already supported
    require!(
        !vault.supported_tokens.iter().any(|t| t.mint == mint),
        VaultError::TokenAlreadySupported
    );

    vault.supported_tokens.push(SupportedToken {
        mint,
        bump: 0,
        total_deposited: 0,
        total_withdrawn: 0,
        is_active: true,
    });

    let clock = Clock::get()?;
    emit!(TokenAddedEvent {
        vault: vault.key(),
        authority: ctx.accounts.authority.key(),
        token_mint: mint,
        vault_token_account: ctx.accounts.vault_token_account.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Added token {} to vault", mint);
    Ok(())
}
