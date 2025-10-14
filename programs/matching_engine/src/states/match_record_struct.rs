use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MatchRecordsStruct {
    pub user_key: [u8; 32],
    pub user_nonce: u128,
    pub order_book_nonce: u128,
    pub bump: u8,
}