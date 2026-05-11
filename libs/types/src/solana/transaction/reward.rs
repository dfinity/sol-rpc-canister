use crate::{Pubkey, RpcError};
use candid::{CandidType, Deserialize};
use serde::Serialize;

/// Represents a reward or penalty applied to an account for fees, rent, voting, or staking activity.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct Reward {
    /// The public key, of the account that received the reward.
    pub pubkey: Pubkey,
    /// Number of reward lamports credited or debited by the account
    pub lamports: i64,
    /// Account balance in lamports after the reward was applied.
    #[serde(rename = "postBalance")]
    pub post_balance: u64,
    /// Type of reward.
    #[serde(rename = "rewardType")]
    pub reward_type: Option<RewardType>,
    /// Vote account commission when the reward was credited, only present for voting and staking
    /// rewards.
    pub commission: Option<u8>,
}

impl TryFrom<solana_transaction_status_client_types::Reward> for Reward {
    type Error = RpcError;

    fn try_from(
        reward: solana_transaction_status_client_types::Reward,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            pubkey: reward.pubkey.parse()?,
            lamports: reward.lamports,
            post_balance: reward.post_balance,
            reward_type: reward.reward_type.map(reward_type_from_status_client),
            commission: reward.commission,
        })
    }
}

impl From<Reward> for solana_transaction_status_client_types::Reward {
    fn from(reward: Reward) -> Self {
        Self {
            pubkey: reward.pubkey.to_string(),
            lamports: reward.lamports,
            post_balance: reward.post_balance,
            reward_type: reward.reward_type.and_then(reward_type_to_status_client),
            commission: reward.commission,
        }
    }
}

/// Enum representing the type of reward granted to an account on the Solana network.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum RewardType {
    /// Reward from transaction fees collected in the block.
    #[serde(rename = "fee")]
    Fee,
    /// Reward from rent fees paid by accounts storing data on-chain.
    #[serde(rename = "rent")]
    Rent,
    /// Reward earned from delegating stake to validators with good performance.
    #[serde(rename = "staking")]
    Staking,
    /// Reward earned for participating in vote transactions to help reach consensus.
    #[serde(rename = "voting")]
    Voting,
    /// Reward returned to an account when its stake is deactivated.
    #[serde(rename = "deactivatedStake")]
    DeactivatedStake,
}

impl From<solana_reward_info::RewardType> for RewardType {
    fn from(reward_type: solana_reward_info::RewardType) -> Self {
        match reward_type {
            solana_reward_info::RewardType::Fee => Self::Fee,
            solana_reward_info::RewardType::Rent => Self::Rent,
            solana_reward_info::RewardType::Staking => Self::Staking,
            solana_reward_info::RewardType::Voting => Self::Voting,
            solana_reward_info::RewardType::DeactivatedStake => Self::DeactivatedStake,
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
            RewardType::DeactivatedStake => Self::DeactivatedStake,
        }
    }
}

fn reward_type_from_status_client(
    reward_type: solana_reward_info_legacy::RewardType,
) -> RewardType {
    match reward_type {
        solana_reward_info_legacy::RewardType::Fee => RewardType::Fee,
        solana_reward_info_legacy::RewardType::Rent => RewardType::Rent,
        solana_reward_info_legacy::RewardType::Staking => RewardType::Staking,
        solana_reward_info_legacy::RewardType::Voting => RewardType::Voting,
    }
}

fn reward_type_to_status_client(
    reward_type: RewardType,
) -> Option<solana_reward_info_legacy::RewardType> {
    match reward_type {
        RewardType::Fee => Some(solana_reward_info_legacy::RewardType::Fee),
        RewardType::Rent => Some(solana_reward_info_legacy::RewardType::Rent),
        RewardType::Staking => Some(solana_reward_info_legacy::RewardType::Staking),
        RewardType::Voting => Some(solana_reward_info_legacy::RewardType::Voting),
        RewardType::DeactivatedStake => None,
    }
}
