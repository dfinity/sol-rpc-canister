//! A demo of a very bare-bones Solana "wallet".
//!
//! The wallet here showcases how Solana addresses can be computed and how Solana transactions
//! can be signed. It is missing several pieces that any production-grade wallet would have,
//! such as error handling, access-control, caching, etc.

use crate::{
    ed25519::Ed25519ExtendedPublicKey,
    state::{lazy_call_ed25519_public_key, read_state},
};
use candid::Principal;
use sol_rpc_client::{
    ed25519::{sign_message, DerivationPath},
    IcRuntime,
};
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use std::fmt::Display;

#[derive(Clone)]
pub struct SolanaAccount {
    pub ed25519_public_key: Pubkey,
    pub derivation_path: DerivationPath,
}

impl SolanaAccount {
    pub fn new_derived_account(
        root_public_key: &Ed25519ExtendedPublicKey,
        derivation_path: DerivationPath,
    ) -> Self {
        let ed25519_public_key = root_public_key
            .derive_public_key(derivation_path.clone())
            .public_key
            .serialize_raw()
            .into();
        Self {
            ed25519_public_key,
            derivation_path,
        }
    }

    pub async fn sign_message(&self, message: &Message) -> Signature {
        sign_message(
            &IcRuntime,
            message,
            read_state(|s| s.ed25519_key_name()).into(),
            Some(&self.derivation_path),
        )
        .await
        .expect("Failed to sign transaction")
    }
}

impl AsRef<Pubkey> for SolanaAccount {
    fn as_ref(&self) -> &Pubkey {
        &self.ed25519_public_key
    }
}

impl Display for SolanaAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            bs58::encode(&self.ed25519_public_key).into_string()
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SolanaWallet {
    owner: Principal,
    root_public_key: Ed25519ExtendedPublicKey,
}

impl SolanaWallet {
    pub async fn new(owner: Principal) -> Self {
        let root_public_key = lazy_call_ed25519_public_key().await;
        Self {
            owner,
            root_public_key,
        }
    }

    pub fn derive_account(&self, derivation_path: DerivationPath) -> SolanaAccount {
        SolanaAccount::new_derived_account(&self.root_public_key, derivation_path)
    }

    pub fn solana_account(&self) -> SolanaAccount {
        self.derive_account(self.owner.as_slice().into())
    }

    pub fn derived_nonce_account(&self) -> SolanaAccount {
        self.derive_account(
            [self.owner.as_slice(), "nonce-account".as_bytes()]
                .concat()
                .as_slice()
                .into(),
        )
    }
}
