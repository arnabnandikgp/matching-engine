use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::errors::ErrorCode;
use crate::states::MatchRecord;

const SETTLEMENT_BOT_PUBKEY: Pubkey =  pubkey!("11111111111111111111111111111111");
// not the vault authority but a separate authority that can only execute settlements.
// the vault authority is the one that can execute deposits and withdrawals which is a pda derived from the main program.

pub fn execute_settlement(
    ctx: Context<ExecuteSettlement>,
    match_id: u64,
    quantity: u64,
    execution_price: u64,
) -> Result<()> {
    // Verify settlement authority (settlement bot)
    require!(
        ctx.accounts.settlement_authority.key() == SETTLEMENT_BOT_PUBKEY,
        ErrorCode::UnauthorizedSettlement
    );
    
    // Prevent double settlement
    require!(
        !ctx.accounts.match_record.is_settled,
        ErrorCode::AlreadySettled
    );
    
    let quote_amount = quantity
        .checked_mul(execution_price)
        .ok_or(ErrorCode::Overflow)?;
    
    // Transfer base tokens: seller → buyer
    let base_transfer_cpi = Transfer {
        from: ctx.accounts.seller_base_vault.to_account_info(),
        to: ctx.accounts.buyer_base_vault.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    
    let vault_auth_bump = ctx.bumps.vault_authority;
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"vault_authority",
        &[vault_auth_bump],
    ]];
    
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            base_transfer_cpi,
            signer_seeds,
        ),
        quantity,
    )?;
    
    // Transfer quote tokens: buyer → seller
    let quote_transfer_cpi = Transfer {
        from: ctx.accounts.buyer_quote_vault.to_account_info(),
        to: ctx.accounts.seller_quote_vault.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            quote_transfer_cpi,
            signer_seeds,
        ),
        quote_amount,
    )?;
    
    ctx.accounts.match_record.is_settled = true;
    ctx.accounts.match_record.settlement_timestamp = Clock::get()?.unix_timestamp;
    
    emit!(SettlementExecutedEvent {
        match_id,
        buyer: ctx.accounts.buyer_base_vault.owner,
        seller: ctx.accounts.seller_base_vault.owner,
        quantity,
        execution_price,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(match_id: u64)]
pub struct ExecuteSettlement<'info> {
    #[account(mut)]
    pub settlement_authority: Signer<'info>,  

    /// CHECK: PDA authority for vault
    #[account(
        seeds = [b"vault_authority"],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    
    #[account(
        init_if_needed,
        payer = settlement_authority,
        space = 8 + MatchRecord::INIT_SPACE,
        seeds = [b"match_record", match_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub match_record: Account<'info, MatchRecord>,
    
    // Buyer vaults
    #[account(mut)]
    pub buyer_base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_quote_vault: Account<'info, TokenAccount>,
    
    // Seller vaults
    #[account(mut)]
    pub seller_base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller_quote_vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct SettlementExecutedEvent {
    pub match_id: u64,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub quantity: u64,
    pub execution_price: u64,
    pub timestamp: i64,
}