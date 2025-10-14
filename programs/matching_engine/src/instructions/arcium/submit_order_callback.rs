use arcium_anchor::prelude::*;
use anchor_lang::prelude::*;
use crate::ID_CONST;
use crate::COMP_DEF_OFFSET_MATCH_ORDERS;
use crate::states::*;
#[callback_accounts("submit_order")]
#[derive(Accounts)]
pub struct SubmitOrderCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH_ORDERS))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub order_account: Account<'info, OrderAccount>,

    #[account(mut)]
    pub global_orderbook: Account<'info, GlobalOrderBookState>,
}