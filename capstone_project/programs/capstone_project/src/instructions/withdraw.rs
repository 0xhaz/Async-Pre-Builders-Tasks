use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::Vault;
use crate::error::VaultError;
use crate::events::TokenWithdrawnEvent;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.authority.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        mut,
        constraint = vault_token_account.owner == vault.key() @ VaultError::InvalidAccountData,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = user_token_account.mint == vault_token_account.mint @ VaultError::InvalidAccountData,
        constraint = user_token_account.owner == user_authority.key() @ VaultError::InvalidAccountData,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    pub user_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    // Phase 1: Read vault state (temporary borrows only)
    require!(!ctx.accounts.vault.paused, VaultError::VaultPaused);

    let token_mint = ctx.accounts.vault_token_account.mint;

    require!(
        ctx.accounts.vault.supported_tokens.iter().any(|t| t.mint == token_mint && t.is_active),
        VaultError::TokenNotSupported
    );

    let vault_balance = ctx.accounts.vault.token_balances
        .iter()
        .find(|b| b.mint == token_mint)
        .map(|b| b.balance)
        .unwrap_or(0);
    require!(vault_balance >= amount, VaultError::InvalidAmount);

    let withdrawal_fee = if amount > 0 {
        (amount as u128 * ctx.accounts.vault.fee_config.withdrawal_fee_bps as u128 / 10000) as u64
    } else {
        0
    };
    let net_withdrawal_amount = amount - withdrawal_fee;

    let authority_key = ctx.accounts.vault.authority;
    let bump = ctx.accounts.vault.bump;

    // Phase 2: CPI - transfer tokens from vault to user (vault PDA signs)
    let seeds = &[b"vault" as &[u8], authority_key.as_ref(), &[bump]];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    token::transfer(cpi_ctx, net_withdrawal_amount)?;

    // Phase 3: Update vault state (mutable borrow)
    let vault = &mut ctx.accounts.vault;

    if let Some(supported_token) = vault.supported_tokens.iter_mut().find(|t| t.mint == token_mint) {
        supported_token.total_withdrawn += net_withdrawal_amount;
    }

    let clock = Clock::get()?;
    if let Some(balance) = vault.token_balances.iter_mut().find(|b| b.mint == token_mint) {
        balance.balance -= net_withdrawal_amount;
        balance.last_updated = clock.unix_timestamp;
    }

    vault.total_value_locked -= net_withdrawal_amount;
    vault.total_fees_collected += withdrawal_fee;

    emit!(TokenWithdrawnEvent {
        vault: vault.key(),
        token_mint,
        amount: net_withdrawal_amount,
        fee_amount: withdrawal_fee,
        recipient: ctx.accounts.user_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Withdrew {} tokens (fee: {})", net_withdrawal_amount, withdrawal_fee);
    Ok(())
}
