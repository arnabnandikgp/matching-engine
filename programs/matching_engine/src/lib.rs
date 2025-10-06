use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;  
// use arcium_anchor::prelude::*;

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

    // pub fn init_match_orders_comp_def(ctx: Context<InitMatchOrdersCompDef>) -> Result<()> {
    //     init_comp_def(ctx.accounts, true, 0, None, None)?;
    //     Ok(())
    // }

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)?;
        Ok(())
    }

    pub fn deposit_to_vault(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
        instructions::deposit_to_vault(ctx, amount)?;
        Ok(())
    }

    pub fn submit_order(ctx: Context<SubmitOrder>, computation_offset: u64, encrypted_order: [u8; 32], pub_key: [u8; 32], nonce: u128) -> Result<()> {
        instructions::submit_order(ctx, computation_offset, encrypted_order, pub_key, nonce)?;
        Ok(())
    }
    
    pub fn trigger_matching(ctx: Context<TriggerMatching>, computation_offset: u64, pub_key: [u8; 32], nonce: u128) -> Result<()> {
        trigger_matching(ctx, computation_offset, pub_key, nonce)?;
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



// #[init_computation_definition_accounts("match_orders", payer)]
// #[derive(Accounts)]
// pub struct InitMatchOrdersCompDef<'info> {
//     #[account(mut)]
//     pub payer: Signer<'info>,
//     #[account(mut, address = derive_mxe_pda!())]
//     pub mxe_account: Box<Account<'info, MXEAccount>>,
//     /// CHECK: comp_def_account, checked by the arcium program.
//     #[account(mut)]
//     pub comp_def_account: UncheckedAccount<'info>,
//     pub arcium_program: Program<'info, Arcium>,
//     pub system_program: Program<'info, System>,
// }
