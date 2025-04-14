use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct Reward {
    pub pubkey: String,
    pub lamports: i64,
    pub post_balance: u64,
    pub reward_type: Option<RewardType>,
    pub commission: Option<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum RewardType {
    Fee,
    Rent,
    Staking,
    Voting,
}