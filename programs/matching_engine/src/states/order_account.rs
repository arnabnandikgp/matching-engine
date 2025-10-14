use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct OrderAccount {
    pub order_id: u64,
    pub user: Pubkey,
    pub order_type: u8,  // 0 = buy, 1 = sell
    pub status: u8,  // 0 = pending, 1 = filled, 2 = cancelled, 3 = partial
    pub locked_amount: u64,
    pub filled_amount: u64,
    pub timestamp: i64,
    pub bump: u8,
}