use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use crate::errors::ErrorCode;
use crate::states::OrderBook;
use crate::instructions::*;
use crate::COMP_DEF_OFFSET_MATCH_ORDERS;
use crate::SignerAccount;

const ORDER_BOOK_SEED: &[u8] = b"order_book";
use crate::ID;
use crate::ID_CONST;

pub fn submit_order(
    ctx: Context<SubmitOrder>,
    computation_offset: u64,
    encrypted_order: [u8; 32],
    pub_key: [u8; 32],
    nonce: u128,
) -> Result<()> {
    let order_book = &mut ctx.accounts.order_book;
    let order_id = order_book.next_order_id;
    order_book.next_order_id = order_book.next_order_id.checked_add(1)
        .ok_or(ErrorCode::OrderIdOverflow)?;

    ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

    let args = vec![
        Argument::ArcisPubkey(pub_key),
        Argument::PlaintextU128(nonce),
        Argument::PlaintextU64(order_id),
        Argument::PlaintextU64(Clock::get()?.unix_timestamp as u64),
        Argument::EncryptedU8(encrypted_order),
    ];

    queue_computation(
        ctx.accounts,
        computation_offset,
        args,
        None,
        vec![SubmitOrderCallback::callback_ix(&[])],
    )?;

    emit!(OrderSubmittedEvent {
        user: ctx.accounts.user.key(),
        order_id,
        computation_offset,
    });

    Ok(())
}

#[queue_computation_accounts("submit_order", user)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct SubmitOrder<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [ORDER_BOOK_SEED],
        bump = order_book.bump,
    )]
    pub order_book: Account<'info, OrderBook>,
    #[account(
        init_if_needed,
        space = 9,
        payer = user,
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

#[event]
pub struct OrderSubmittedEvent {
    pub user: Pubkey,
    pub order_id: u64,
    pub computation_offset: u64,
}
