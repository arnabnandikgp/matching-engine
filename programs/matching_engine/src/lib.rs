use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_MATCH_ORDERS: u32 = comp_def_offset("match_orders");

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
        let (match_result_encrypted, orderbook_encrypted) = match output {
            ComputationOutputs::Success(MatchOrdersOutput {
                field_0: MatchOrdersOutputStruct0 {
                    field_0: match_result,    // Enc<Shared, MatchResult>
                    field_1: orderbook,       // Enc<Mxe, OrderBook>
                },
            }) => (match_result, orderbook),
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };
    
        let new_orderbook_nonce = orderbook_encrypted.nonce;
        let orderbook_ciphertext = orderbook_encrypted.ciphertexts;
        
        let global_orderbook = &mut ctx.accounts.global_orderbook;
        global_orderbook.orderbook_nonce = new_orderbook_nonce;
        
        // Copy orderbook ciphertext to account storage
        for (i, chunk) in orderbook_ciphertext.iter().enumerate() {
            let start = i * 32;
            let end = start + 32;
            if end <= global_orderbook.orderbook_data.len() {
                global_orderbook.orderbook_data[start..end].copy_from_slice(chunk);
            }
        }
        
        // Extract match result data (encrypted for backend)
        let match_nonce = match_result_encrypted.nonce;
        let match_ciphertext = match_result_encrypted.ciphertexts;
        
        // Update metadata
        let timestamp = Clock::get()?.unix_timestamp;
        
        emit!(MatchResultEvent {
            match_ciphertext,         // Backend decrypts this with match_nonce
            match_nonce,              // Backend needs this nonce!
            orderbook_nonce: new_orderbook_nonce,
            timestamp,
        });
    
        msg!("Matches computed. Backend can now decrypt match results using emitted nonce.");
        
        Ok(())
    
    }

    #[arcium_callback(encrypted_ix = "submit_order", network = "localnet")]
    pub fn submit_order_callback(
        ctx: Context<SubmitOrderCallback>,
        output: ComputationOutputs<SubmitOrderOutput>,
    ) -> Result<()> {
        let (orderbook_encrypted, success, buy_count, sell_count) = match output {
            ComputationOutputs::Success(SubmitOrderOutput {
                field_0: SubmitOrderOutputStruct0 {
                    field_0: orderbook,
                    field_1: success,
                    field_2: buy_count,
                    field_3: sell_count,
                },
            }) => (orderbook, success, buy_count, sell_count),
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        let new_orderbook_nonce = orderbook_encrypted.nonce;
        let orderbook_ciphertext = orderbook_encrypted.ciphertexts;
        
        let global_orderbook = &mut ctx.accounts.global_orderbook;
        global_orderbook.orderbook_nonce = new_orderbook_nonce;
        
        // Copy ciphertext to account storage
        for (i, chunk) in orderbook_ciphertext.iter().enumerate() {
            let start = i * 32;
            let end = start + 32;
            if end <= global_orderbook.orderbook_data.len() {
                global_orderbook.orderbook_data[start..end].copy_from_slice(chunk);
            }
        }
        
        global_orderbook.total_orders_processed += 1;
        
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

#[event]
pub struct MatchResultEvent {
    pub match_ciphertext: [[u8; 32]; 336],  // Encrypted Enc<Shared, MatchResult>
    pub match_nonce: u128,                    //  Backend needs this to decrypt!
    pub orderbook_nonce: u128,                // New orderbook nonce
    pub timestamp: i64,
}