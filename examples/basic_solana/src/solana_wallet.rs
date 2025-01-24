//! A demo of a very bare-bones Solana "wallet".
//!
//! The wallet here showcases how Solana addresses can be computed and how Solana transactions
//! can be signed. It is missing several pieces that any production-grade wallet would have,
//! such as error handling, access-control, caching, etc.

use crate::{ed25519::Ed25519ExtendedPublicKey, state::lazy_call_ed25519_public_key};
use candid::Principal;
use ic_crypto_ed25519::{DerivationIndex, DerivationPath};
use serde_bytes::ByteBuf;
use std::fmt::Display;

pub struct SolanaAccount {
    ed25519_public_key: [u8; 32],
}

impl From<&Ed25519ExtendedPublicKey> for SolanaAccount {
    fn from(public_key: &Ed25519ExtendedPublicKey) -> Self {
        Self {
            ed25519_public_key: public_key.public_key.serialize_raw()
        }
    }
}

impl Display for SolanaAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", bs58::encode(&self.ed25519_public_key).into_string())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SolanaWallet {
    owner: Principal,
    derived_public_key: Ed25519ExtendedPublicKey,
}

impl SolanaWallet {
    pub async fn new(owner: Principal) -> Self {
        let derived_public_key = derive_public_key(&owner, &lazy_call_ed25519_public_key().await);
        Self {
            owner,
            derived_public_key,
        }
    }

    pub fn solana_account(&self) -> SolanaAccount {
        SolanaAccount::from(&self.derived_public_key)
    }
}

fn derive_public_key(owner: &Principal, public_key: &Ed25519ExtendedPublicKey) -> Ed25519ExtendedPublicKey {
    let derivation_path = DerivationPath::new(
        derivation_path(owner)
            .into_iter()
            .map(DerivationIndex)
            .collect(),
    );
    public_key.derive_new_public_key(&derivation_path)
}

fn derivation_path(owner: &Principal) -> Vec<Vec<u8>> {
    const SCHEMA_V1: u8 = 1;
    [
        ByteBuf::from(vec![SCHEMA_V1]),
        ByteBuf::from(owner.as_slice().to_vec()),
    ]
        .iter()
        .map(|x| x.to_vec())
        .collect()
}