use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct SubmitOrderStruct {
    pub order_nonce: u128,
    pub caller_enc_pubkey: [u8; 32],
    pub order_book_nonce: u128,
    pub bump: u8,
}
