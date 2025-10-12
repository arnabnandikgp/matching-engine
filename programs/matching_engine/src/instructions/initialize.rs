use anchor_lang::prelude::*;
const ORDER_BOOK_STATE_SEED: &[u8] = b"order_book_state";
use crate::{states::OrderBookState};


pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let order_book_state = &mut ctx.accounts.order_book_state;
    order_book_state.authority = ctx.accounts.authority.key();
    order_book_state.last_match_timestamp = Clock::get()?.unix_timestamp;
    order_book_state.bump = ctx.bumps.order_book_state;
    Ok(())
}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + OrderBookState::INIT_SPACE,
        seeds = [ORDER_BOOK_STATE_SEED],
        bump
    )]
    pub order_book_state: Account<'info, OrderBookState>,
    pub system_program: Program<'info, System>,
}
