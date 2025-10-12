use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct OrderBookState {
    pub authority: Pubkey,
    pub last_match_timestamp: i64,
    pub bump: u8,
}
