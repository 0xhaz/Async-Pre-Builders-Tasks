use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{TokenBalance, Vault};
use crate::error::VaultError;
use crate::events::TokenDepositedEvent;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.authority.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = vault_token_account.owner == vault.key() @ VaultError::InvalidAccountData,
        constraint = vault_token_account.mint == user_token_account.mint @ VaultError::InvalidAccountData,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    require!(!vault.paused, VaultError::VaultPaused);

    let token_mint = ctx.accounts.user_token_account.mint;

    // Check if token is supported
    require!(
        vault.supported_tokens.iter().any(|t| t.mint == token_mint && t.is_active),
        VaultError::TokenNotSupported
    );

    // Calculate fees
    let deposit_fee = if amount > 0 {
        (amount as u128 * vault.fee_config.deposit_fee_bps as u128 / 10000) as u64
    } else {
        0
    };
    let net_deposit_amount = amount - deposit_fee;

    // Transfer tokens from user to vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: ctx.accounts.user_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, net_deposit_amount)?;

    // Update supported token totals
    if let Some(supported_token) = vault.supported_tokens.iter_mut().find(|t| t.mint == token_mint) {
        supported_token.total_deposited += net_deposit_amount;
    }

    // Update token balance
    let clock = Clock::get()?;
    let balance_index = vault.token_balances.iter().position(|b| b.mint == token_mint);
    if let Some(index) = balance_index {
        vault.token_balances[index].balance += net_deposit_amount;
        vault.token_balances[index].last_updated = clock.unix_timestamp;
    } else {
        vault.token_balances.push(TokenBalance {
            mint: token_mint,
            balance: net_deposit_amount,
            last_updated: clock.unix_timestamp,
        });
    }

    vault.total_value_locked += net_deposit_amount;
    vault.total_fees_collected += deposit_fee;

    emit!(TokenDepositedEvent {
        vault: vault.key(),
        token_mint,
        amount: net_deposit_amount,
        fee_amount: deposit_fee,
        depositor: ctx.accounts.user_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Deposited {} tokens (fee: {})", net_deposit_amount, deposit_fee);
    Ok(())
}
