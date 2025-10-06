use anchor_lang::prelude::*;
const ORDER_BOOK_SEED: &[u8] = b"order_book";
use crate::states::OrderBook;


pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let order_book = &mut ctx.accounts.order_book;
    order_book.authority = ctx.accounts.authority.key();
    order_book.next_order_id = 0;
    order_book.last_match_timestamp = Clock::get()?.unix_timestamp;
    order_book.bump = ctx.bumps.order_book;
    Ok(())
}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + OrderBook::INIT_SPACE,
        seeds = [ORDER_BOOK_SEED],
        bump
    )]
    pub order_book: Account<'info, OrderBook>,
    pub system_program: Program<'info, System>,
}
