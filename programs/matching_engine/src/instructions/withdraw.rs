use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::states::VaultState;
use crate::errors::ErrorCode;

const VAULT_SEED: &[u8] = b"vault";

pub fn withdraw_from_vault(
    ctx: Context<WithdrawFromVault>,
    amount: u64,
) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;    
    let vault = &mut ctx.accounts.vault;
    let available = vault.amount.checked_sub(vault_state.locked_amount)
    .ok_or(ErrorCode::InsufficientBalance)?;
    require!(
        available >= amount,
        ErrorCode::InsufficientBalance
    );

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    emit!(WithdrawEvent {
        user: ctx.accounts.user.key(),
        amount,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawFromVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        constraint = user_token_account.owner == user.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [VAULT_SEED, vault.mint.as_ref()],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,
    /// CHECK: PDA authority for vault
    #[account(
        seeds = [b"vault_authority"],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [VAULT_SEED, vault.mint.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    pub token_program: Program<'info, Token>,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub amount: u64,
}