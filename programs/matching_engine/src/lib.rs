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

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)?;
        Ok(())
    }

    pub fn deposit_to_vault(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
        instructions::deposit_to_vault(ctx, amount)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "match_orders", network = "localnet")]
    pub fn match_orders_callback(
        ctx: Context<MatchOrdersCallback>,
        output: ComputationOutputs<MatchOrdersOutput>,
    ) -> Result<()> {
        let o = match output {
            ComputationOutputs::Success(MatchOrdersOutput {
                field_0:
                    MatchOrdersOutputStruct0 {
                        field_0: deck,
                        field_1: dealer_hand,
                    },
            }) => (deck, dealer_hand),
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        // emit!(EncryptedMatchesEvent {
        //     computation_offset: ctx.accounts.comp_def_account.key(),
        //     ciphertext: matches.ciphertexts,  // Raw encrypted bytes
        //     nonce: matches.nonce,
        //     timestamp: Clock::get()?.unix_timestamp,
        // });

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
        let o = match output {
            ComputationOutputs::Success(SubmitOrderOutput {
                field_0:
                    SubmitOrderOutputStruct0 {
                        field_0: success,
                        field_1: buy_count,
                        field_2: sell_count,
                    },
            }) => (success, buy_count, sell_count),
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        let success: bool = o.0;
        let buy_count: u8 = o.1;
        let sell_count: u8 = o.2;

        if success {
            ctx.accounts.order_account.status = 1; // processing
        } else {
            ctx.accounts.order_account.status = 2; // cancelled
        }
        // will be used by the cranker to trigger the matching
        emit!(SubmitOrderEvent {
            buy_count,
            sell_count,
        });

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
pub struct SubmitOrderEvent {
    pub buy_count: u8,
    pub sell_count: u8,
}