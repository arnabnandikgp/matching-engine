use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use crate::errors::ErrorCode;

#[arcium_callback(encrypted_ix = "match_orders", network = "localnet")]
pub fn match_orders_callback(
    ctx: Context<MatchOrdersCallback>,
    output: ComputationOutputs<MatchOrdersOutput>,
) -> Result<()> {
    let matches = match output {
        ComputationOutputs::Success(MatchOrdersOutput { field_0 }) => field_0,
        _ => return Err(ErrorCode::AbortedComputation.into()),
    };


    for i in 0..matches.num_matches as usize {
        if i >= 10 {
            break;
        }

        let buyer_id = Pubkey::new_from_array(matches.buyer_ids[i]);
        let seller_id = Pubkey::new_from_array(matches.seller_ids[i]);
        let base_mint = Pubkey::new_from_array(matches.base_mints[i]);
        let quote_mint = Pubkey::new_from_array(matches.quote_mints[i]);
        
        emit!(TradeExecutedEvent {
            match_id: matches.match_ids[i],
            buyer: buyer_id,
            seller: seller_id,
            base_mint,
            quote_mint,
            quantity: matches.quantities[i],
            execution_price: matches.execution_prices[i],
        });
    // }

    Ok(())
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
}


#[event]
pub struct TradeExecutedEvent {
    pub match_id: u64,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub quantity: u64,
    pub execution_price: u64,
}
