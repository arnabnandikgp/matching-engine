use anchor_lang::prelude::*;

// #[account]
// #[derive(InitSpace)]
// pub struct OrderBookState {
//     // pub authority: Pubkey,
    
//     // // Encrypted OrderBook stored on-chain
//     // // OrderBook = 5 Orders (buy) + 1 u8 (buy_count) + 5 Orders (sell) + 1 u8 (sell_count)
//     // // Each Order = 37 ciphertexts (order_id + user_pubkey[32] + amount + price + order_type + timestamp)
//     // // Total: (5*37) + 1 + (5*37) + 1 = 372 ciphertexts * 32 bytes = 11,904 bytes
//     // // pub orderbook_data: [u8; 11904],   // Encrypted OrderBook ciphertexts
//     // pub orderbook_data: [[u8; 32]; 21],
//     // pub orderbook_nonce: u128,  
    
//     // // Backend encryption key (for receiving match results)
//     // pub backend_pubkey: [u8; 32],      // Backend's x25519 public key
    
//     // // Metadata
//     // pub last_match_timestamp: i64,
//     // pub total_orders_processed: u64,
//     // pub total_matches: u64,

//     // // token pair mint
//     // pub base_mint: Pubkey,
//     // pub quote_mint: Pubkey,
    
//     // pub bump: u8,

//     pub authority: Pubkey,              //32
//     pub orderbook_data: [[u8; 32]; 41], // 1312
//     pub orderbook_nonce: u128,          // 16
//     pub backend_pubkey: [u8; 32],       // 32
//     pub base_mint: Pubkey,              // 32
//     pub quote_mint: Pubkey,             // 32   
//     pub last_match_timestamp: i64,      // 8
//     pub total_orders_processed: u64,    // 8
//     pub total_matches: u64,             // 8  
//     pub bump: u8,                       // 1
// }


#[account]
#[derive(InitSpace)]
pub struct OrderBookState {
    pub authority: Pubkey,              // 32
    pub orderbook_data: [[u8; 32]; 52], // 1312
    pub orderbook_nonce: u128,          // 16
    pub backend_pubkey: [u8; 32],       // 32
    pub base_mint: Pubkey,              // 32
    pub quote_mint: Pubkey,             // 32
    pub last_match_timestamp: i64,      // 8
    pub total_orders_processed: u64,    // 8
    pub total_matches: u64,             // 8
    pub bump: u8,                       // 1
}
// Total: 1481 bytes
