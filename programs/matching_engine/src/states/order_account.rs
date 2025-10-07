use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct OrderAccount {
    pub order_id: u64,
    pub user: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub order_type: u8,  // 0 = buy, 1 = sell
    pub amount: u64,
    pub price: u64,
    pub locked_amount: u64,  // How much is locked in vault
    pub status: u8,  // 0 = pending, 1 = filled, 2 = cancelled, 3 = partial
    pub filled_amount: u64,
    pub timestamp: i64,
    pub bump: u8,
}