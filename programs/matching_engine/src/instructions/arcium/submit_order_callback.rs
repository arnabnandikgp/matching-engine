use arcium_anchor::prelude::*;
use crate::states::SubmitOrderCallback;


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


#[callback_accounts("submit_order")]
#[derive(Accounts)]
pub struct SubmitOrderCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH_ORDERS))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
}