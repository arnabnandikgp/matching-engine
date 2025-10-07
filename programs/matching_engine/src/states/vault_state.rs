use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,           // Points to TokenAccount
    pub locked_amount: u64,       // ← Track locked here!
    pub num_active_orders: u16,   // Bonus: track active orders
    pub bump: u8,
}