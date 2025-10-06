use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct OrderBook {
    pub authority: Pubkey,
    pub next_order_id: u64,
    pub last_match_timestamp: i64,
    pub bump: u8,
}
