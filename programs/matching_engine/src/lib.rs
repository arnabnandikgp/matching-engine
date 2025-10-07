use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_MATCH_ORDERS: u32 = comp_def_offset("match_orders");

declare_id!("DQ5MR2aPD9sPBN9ukVkhwrAn8ADxpkAE5AHUnXxKEvn1");

pub mod instructions;
pub mod states;
pub use instructions::*;
pub use states::*;
pub mod errors;
pub use errors::*;

#[arcium_program]
pub mod matching_engine {
    use super::*;

    pub fn init_match_orders_comp_def(ctx: Context<InitMatchOrdersCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)?;
        Ok(())
    }

    pub fn deposit_to_vault(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
        instructions::deposit_to_vault(ctx, amount)?;
        Ok(())
    }

    pub fn submit_order(
        ctx: Context<SubmitOrder>,
        computation_offset: u64,
        encrypted_order: [u8; 32],
        pub_key: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        instructions::submit_order(ctx, computation_offset, encrypted_order, pub_key, nonce)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "match_orders", network = "localnet")]
    pub fn match_orders_callback(
        ctx: Context<MatchOrdersCallback>,
        output: ComputationOutputs<MatchOrdersOutput>,
    ) -> Result<()> {
        let matches = match output {
            ComputationOutputs::Success(MatchOrdersOutput { field_0 }) => field_0,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        // TODO: Handle matches

        // for i in 0..matches.num_matches as usize {
        //     if i >= 10 {
        //         break;
        //     }

        //     let buyer_id = Pubkey::new_from_array(matches.buyer_ids[i]);
        //     let seller_id = Pubkey::new_from_array(matches.seller_ids[i]);
        //     let base_mint = Pubkey::new_from_array(matches.base_mints[i]);
        //     let quote_mint = Pubkey::new_from_array(matches.quote_mints[i]);

        //     emit!(TradeExecutedEvent {
        //         match_id: matches.match_ids[i],
        //         buyer: buyer_id,
        //         seller: seller_id,
        //         base_mint,
        //         quote_mint,
        //         quantity: matches.quantities[i],
        //         execution_price: matches.execution_prices[i],
        //     });
        // }

        Ok(())
    }

    pub fn trigger_matching(
        ctx: Context<TriggerMatching>,
        computation_offset: u64,
        pub_key: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        instructions::trigger_matching(ctx, computation_offset, pub_key, nonce)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "submit_order", network = "localnet")]
    pub fn submit_order_callback(
        ctx: Context<SubmitOrderCallback>,
        output: ComputationOutputs<SubmitOrderOutput>,
    ) -> Result<()> {
        let _result = match output {
            ComputationOutputs::Success(SubmitOrderOutput { field_0 }) => field_0,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        Ok(())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("Cluster not set")]
    ClusterNotSet,
}
