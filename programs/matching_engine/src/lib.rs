use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_MATCH_ORDERS: u32 = comp_def_offset("match_orders");
const COMP_DEF_OFFSET_SUBMIT_ORDER: u32 = comp_def_offset("submit_order");

declare_id!("DQ5MR2aPD9sPBN9ukVkhwrAn8ADxpkAE5AHUnXxKEvn1");

pub mod instructions;
pub mod states;
pub use instructions::*;
pub use states::*;
pub mod errors;
pub use errors::ErrorCode;

#[arcium_program]
pub mod matching_engine {
    use super::*;
    use crate::errors::ErrorCode;

    pub fn init_submit_order_comp_def(ctx: Context<InitSubmitOrderCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }

    pub fn init_match_orders_comp_def(ctx: Context<InitMatchOrdersCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }
    pub fn initialize_vault(ctx: Context<InitializeUserVault>) -> Result<()> {
        instructions::initialize_user_vault(ctx)?;
        Ok(())
    }

    pub fn initialize(ctx: Context<Initialize>, backend_pubkey: [u8; 32], base_mint: Pubkey, quote_mint: Pubkey) -> Result<()> {
        instructions::initialize(ctx, backend_pubkey, base_mint, quote_mint)?;
        Ok(())
    }

    pub fn deposit_to_vault(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
        instructions::deposit_to_vault(ctx, amount)?;
        Ok(())
    }

    pub fn submit_order(ctx: Context<SubmitOrder>, amount: u64, price: u64, order_type: u8, computation_offset: u64, order_id: u64, user_enc_pubkey: [u8; 32], order_nonce: u128) -> Result<()> {
        instructions::submit_order(ctx, amount, price, order_type, computation_offset, order_id, user_enc_pubkey, order_nonce)?;
        Ok(())
    }

    pub fn trigger_matching(
        ctx: Context<TriggerMatching>,
        computation_offset: u64,
    ) -> Result<()> {
        instructions::trigger_matching(ctx, computation_offset)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "match_orders", network = "localnet")]
    pub fn match_orders_callback(
        ctx: Context<MatchOrdersCallback>,
        output: ComputationOutputs<MatchOrdersOutput>,
    ) -> Result<()> {
        // Process the output without storing large structs on stack
        match output {
            ComputationOutputs::Success(MatchOrdersOutput {
                field_0: MatchOrdersOutputStruct0 {
                    field_0: match_result_encrypted,    // Enc<Shared, MatchResult>
                    field_1: orderbook_encrypted,       // Enc<Mxe, OrderBook>
                },
            }) => {
                let new_orderbook_nonce = orderbook_encrypted.nonce;
                let orderbook_ciphertext = &orderbook_encrypted.ciphertexts;
                
                let orderbook_state = &mut ctx.accounts.orderbook_state;
                orderbook_state.orderbook_nonce = new_orderbook_nonce;
                
                // Copy orderbook ciphertext to account storage
                for (i, chunk) in orderbook_ciphertext.iter().enumerate() {
                    let start = i * 32;
                    let end = start + 32;
                    if end <= orderbook_state.orderbook_data.len() {
                        orderbook_state.orderbook_data[start..end].copy_from_slice(chunk);
                    }
                }
                
                // Extract match result data (encrypted for backend)
                let match_nonce = match_result_encrypted.nonce;
                let match_ciphertext = &match_result_encrypted.ciphertexts;
                
                // Update metadata
                let timestamp = Clock::get()?.unix_timestamp;
                
                emit!(MatchResultEvent {
                    match_ciphertext: Box::new(match_ciphertext.clone()),
                    match_nonce,
                    orderbook_nonce: new_orderbook_nonce,
                    timestamp,
                });
            
                msg!("Matches computed. Backend can now decrypt match results using emitted nonce.");
            },
            _ => return Err(ErrorCode::AbortedComputation.into()),
        }
        
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "submit_order", network = "localnet")]
    pub fn submit_order_callback(
        ctx: Context<SubmitOrderCallback>,
        output: ComputationOutputs<SubmitOrderOutput>,
    ) -> Result<()> {
        // Process the output without storing large structs on stack
        match output {
            ComputationOutputs::Success(SubmitOrderOutput {
                field_0: SubmitOrderOutputStruct0 {
                    field_0: orderbook_encrypted,
                    field_1: success,
                    field_2: buy_count,
                    field_3: sell_count,
                },
            }) => {
                let new_orderbook_nonce = orderbook_encrypted.nonce;
                let orderbook_ciphertext = &orderbook_encrypted.ciphertexts;
                
                let orderbook_state = &mut ctx.accounts.orderbook_state;
                orderbook_state.orderbook_nonce = new_orderbook_nonce;
                
                // Copy ciphertext to account storage
                for (i, chunk) in orderbook_ciphertext.iter().enumerate() {
                    let start = i * 32;
                    let end = start + 32;
                    if end <= orderbook_state.orderbook_data.len() {
                        orderbook_state.orderbook_data[start..end].copy_from_slice(chunk);
                    }
                }
                
                orderbook_state.total_orders_processed += 1;
                
                let order_account = &mut ctx.accounts.order_account;
                if success {
                    order_account.status = 1; // Processing (added to orderbook)
                } else {
                    order_account.status = 2; // Rejected (orderbook full)
                }

                // Emit event for cranker
                emit!(OrderProcessedEvent {
                    order_id: order_account.order_id,
                    success,
                    buy_count,
                    sell_count,
                    orderbook_nonce: new_orderbook_nonce,
                });

                msg!("Order processed. Success: {}, Buy Count: {}, Sell Count: {}", success, buy_count, sell_count);
            },
            _ => return Err(ErrorCode::AbortedComputation.into()),
        }
        
        Ok(())
    }
    pub fn withdraw_from_vault(ctx: Context<WithdrawFromVault>, amount: u64) -> Result<()> {
        instructions::withdraw_from_vault(ctx, amount)?;
        Ok(())
    }

    pub fn execute_settlement(ctx: Context<ExecuteSettlement>, match_id: u64, quantity: u64, execution_price: u64) -> Result<()> {
        instructions::execute_settlement(ctx, match_id, quantity, execution_price)?;
        Ok(())
    }
}

#[event]
pub struct OrderProcessedEvent {
    pub order_id: u64,
    pub success: bool,
    pub buy_count: u8,
    pub sell_count: u8,
    pub orderbook_nonce: u128,
}

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
    pub orderbook_state: Account<'info, OrderBookState>,
}

#[callback_accounts("submit_order")]
#[derive(Accounts)]
pub struct SubmitOrderCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_SUBMIT_ORDER))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub order_account: Account<'info, OrderAccount>,
    #[account(mut)]
    pub orderbook_state: Account<'info, OrderBookState>,
}

#[event]
pub struct MatchResultEvent {
    pub match_ciphertext: Box<[[u8; 32]; 202]>,  // Updated to match actual size
    pub match_nonce: u128,                    //  Backend needs this to decrypt!
    pub orderbook_nonce: u128,                // New orderbook nonce
    pub timestamp: i64,
}