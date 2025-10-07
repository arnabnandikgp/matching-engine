use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MatchRecord {
    pub match_id: u64,
    pub is_settled: bool,
    pub settlement_timestamp: i64,
}