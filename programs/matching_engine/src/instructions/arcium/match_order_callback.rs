use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use crate::ID_CONST;
use crate::COMP_DEF_OFFSET_MATCH_ORDERS;
use crate::states::GlobalOrderBookState;
#[callback_accounts("match_orders")]
#[derive(Accounts)]
pub struct MatchOrdersCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH_ORDERS))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    
    #[account(mut)]
    pub global_orderbook: Account<'info, GlobalOrderBookState>,
}

#[event]
pub struct EncryptedMatchesEvent {
    pub computation_offset: Pubkey,
    pub ciphertext: [[u8; 32]; 1466],  // Encrypted match data
    pub nonce: u128,
    pub timestamp: i64,
}
