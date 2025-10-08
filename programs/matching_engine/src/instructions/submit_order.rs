use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use anchor_spl::token::TokenAccount;
use arcium_anchor::prelude::*;
use crate::errors::ErrorCode;
use crate::states::*;
use crate::instructions::*;
use crate::COMP_DEF_OFFSET_MATCH_ORDERS;
use crate::SignerAccount;
const VAULT_SEED: &[u8] = b"vault";

const ORDER_BOOK_STATE_SEED: &[u8] = b"order_book_state";
use crate::ID;
use crate::ID_CONST;

pub fn submit_order(
    ctx: Context<SubmitOrder>,
    amount: u64,
    price: u64,
    order_type: u8,  // 0 = buy, 1 = sell
    computation_offset: u64,
    encrypted_order: [u8; 32],
    pub_key: [u8; 32],
    nonce: u128,
) -> Result<()> {
    let order_book = &mut ctx.accounts.order_book_state;
    let order_id = order_book.next_order_id;
    order_book.next_order_id = order_book.next_order_id.checked_add(1)
        .ok_or(ErrorCode::OrderIdOverflow)?;
    
    // Calculate required funds
    let locked_amount = if order_type == 0 {
        // Buy: lock quote tokens (USDC)
        amount.checked_mul(price).ok_or(ErrorCode::Overflow)?
    } else {
        // Sell: lock base tokens (SOL)
        amount
    };
    
    // Check vault has sufficient funds
    let vault = if order_type == 0 {
        &ctx.accounts.quote_vault
    } else {
        &ctx.accounts.base_vault
    };
    
    // How much is locked in vault
    let locked_total = ctx.accounts.vault_state.locked_amount;
    
    let available = vault.amount.checked_sub(locked_total)
        .ok_or(ErrorCode::InsufficientBalance)?;
    
    require!(
        available >= locked_amount,
        ErrorCode::InsufficientBalance
    );
    
    // Initialize order account
    let order_account = &mut ctx.accounts.order_account;
    order_account.order_id = order_id;
    order_account.user = ctx.accounts.user.key();
    order_account.base_mint = ctx.accounts.base_mint.key();
    order_account.quote_mint = ctx.accounts.quote_mint.key();
    order_account.order_type = order_type;
    order_account.amount = amount;
    order_account.price = price;
    order_account.locked_amount = locked_amount;
    order_account.status = 0; // Pending
    order_account.filled_amount = 0;
    order_account.timestamp = Clock::get()?.unix_timestamp;
    order_account.bump = ctx.bumps.order_account;

    // Updating the vault state
    ctx.accounts.vault_state.locked_amount = ctx.accounts.vault_state.locked_amount.checked_add(locked_amount)
        .ok_or(ErrorCode::Overflow)?;
    ctx.accounts.vault_state.num_active_orders = ctx.accounts.vault_state.num_active_orders.checked_add(1)
        .ok_or(ErrorCode::Overflow)?;
    
    // Queue to MXE
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
        locked_amount,
        vault: vault.key(),
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
        seeds = [VAULT_SEED, base_mint.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub base_vault: Account<'info, TokenAccount>,
    // User's quote token vault (e.g., USDC)
    #[account(
        mut,
        seeds = [VAULT_SEED, quote_mint.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub quote_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [ORDER_BOOK_STATE_SEED],
        bump = order_book_state.bump,
    )]
    pub order_book_state: Account<'info, OrderBookState>,

    #[account(mut)]
    pub base_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub quote_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = user,
        space = 8 + OrderAccount::INIT_SPACE,
        seeds = [
            b"order",
            order_book_state.next_order_id.to_le_bytes().as_ref(),
            user.key().as_ref(),
        ],
        bump,
    )]
    pub order_account: Account<'info, OrderAccount>,

    #[account(
        mut,
        seeds = [VAULT_SEED, base_mint.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,

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
    pub locked_amount: u64,
    pub vault: Pubkey,
}
