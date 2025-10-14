use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct OrderBookState {
    pub authority: Pubkey,
    
    // Encrypted OrderBook stored on-chain
    pub orderbook_data: [u8; 1302],   // Serialized Enc<Mxe, OrderBook> ciphertext
    pub orderbook_nonce: u128,  
    
    // Backend encryption key (for receiving match results)
    pub backend_pubkey: [u8; 32],      // Backend's x25519 public key
    
    // Metadata
    pub last_match_timestamp: i64,
    pub total_orders_processed: u64,
    pub total_matches: u64,

    // token pair mint
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    
    pub bump: u8,
}
