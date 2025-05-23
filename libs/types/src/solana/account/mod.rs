use crate::Pubkey;
use candid::{CandidType, Deserialize};
use serde::Serialize;

/// Solana [account](https://solana.com/docs/references/terminology#account) information.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct Account {
    /// Number of lamports assigned to this account.
    pub lamports: u64,
    /// Data associated with the account.
    pub data: Vec<u8>,
    /// The program this account has been assigned to.
    pub owner: Pubkey,
    /// Boolean indicating if the account contains a program (and is strictly read-only).
    pub executable: bool,
    /// The epoch at which this account will next owe rent.
    #[serde(rename = "rentEpoch")]
    pub rent_epoch: u64,
}

impl From<solana_account::Account> for Account {
    fn from(account: solana_account::Account) -> Self {
        Account {
            lamports: account.lamports,
            data: account.data,
            owner: account.owner.into(),
            executable: account.executable,
            rent_epoch: account.rent_epoch,
        }
    }
}

impl From<Account> for solana_account::Account {
    fn from(account: Account) -> Self {
        solana_account::Account {
            lamports: account.lamports,
            data: account.data,
            owner: account.owner.into(),
            executable: account.executable,
            rent_epoch: account.rent_epoch,
        }
    }
}
