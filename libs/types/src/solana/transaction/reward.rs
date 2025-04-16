use crate::Pubkey;
use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct Reward {
    pub pubkey: Pubkey,
    pub lamports: i64,
    pub post_balance: u64,
    pub reward_type: Option<RewardType>,
    pub commission: Option<u8>,
}

impl From<solana_transaction_status_client_types::Reward> for Reward {
    fn from(reward: solana_transaction_status_client_types::Reward) -> Self {
        Self {
            pubkey: reward.pubkey,
            lamports: reward.lamports,
            post_balance: reward.post_balance,
            reward_type: reward.reward_type.map(Into::into),
            commission: reward.commission,
        }
    }
}

impl From<Reward> for solana_transaction_status_client_types::Reward {
    fn from(reward: Reward) -> Self {
        Self {
            pubkey: reward.pubkey,
            lamports: reward.lamports,
            post_balance: reward.post_balance,
            reward_type: reward.reward_type.map(Into::into),
            commission: reward.commission,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum RewardType {
    Fee,
    Rent,
    Staking,
    Voting,
}

impl From<solana_reward_info::RewardType> for RewardType {
    fn from(reward_type: solana_reward_info::RewardType) -> Self {
        match reward_type {
            solana_reward_info::RewardType::Fee => Self::Fee,
            solana_reward_info::RewardType::Rent => Self::Rent,
            solana_reward_info::RewardType::Staking => Self::Staking,
            solana_reward_info::RewardType::Voting => Self::Voting,
        }
    }
}

impl From<RewardType> for solana_reward_info::RewardType {
    fn from(reward_type: RewardType) -> Self {
        match reward_type {
            RewardType::Fee => Self::Fee,
            RewardType::Rent => Self::Rent,
            RewardType::Staking => Self::Staking,
            RewardType::Voting => Self::Voting,
        }
    }
}
