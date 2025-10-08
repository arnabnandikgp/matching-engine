use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use crate::errors::ErrorCode;
use crate::states::OrderBookState;
use crate::instructions::*;
use crate::COMP_DEF_OFFSET_MATCH_ORDERS;
use crate::SignerAccount;

const ORDER_BOOK_STATE_SEED: &[u8] = b"order_book_state";
use crate::ID;
use crate::ID_CONST;


pub fn trigger_matching(
    ctx: Context<TriggerMatching>,
    computation_offset: u64,
    pub_key: [u8; 32],
    nonce: u128,
) -> Result<()> {
    let order_book = &mut ctx.accounts.order_book_state;
    let current_time = Clock::get()?.unix_timestamp;
    
    require!(
        current_time >= order_book.last_match_timestamp + 15,
        ErrorCode::MatchingTooFrequent
    );

    order_book.last_match_timestamp = current_time;
    ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

    let args = vec![
        Argument::ArcisPubkey(pub_key),
        Argument::PlaintextU128(nonce),
    ];

    queue_computation(
        ctx.accounts,
        computation_offset,
        args,
        None,
        vec![MatchOrdersCallback::callback_ix(&[])],
    )?;

    Ok(())
}


#[queue_computation_accounts("match_orders", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct TriggerMatching<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [ORDER_BOOK_STATE_SEED],
        bump = order_book_state.bump,
    )]
    pub order_book_state: Account<'info, OrderBookState>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, SignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!())]
    /// CHECK: mempool_account, checked by the arcium program.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!())]
    /// CHECK: executing_pool, checked by the arcium program.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset))]
    /// CHECK: computation_account, checked by the arcium program.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH_ORDERS))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}