//! A demo of a very bare-bones Solana "wallet".
//!
//! The wallet here showcases how Solana addresses can be computed and how Solana transactions
//! can be signed. It is missing several pieces that any production-grade wallet would have,
//! such as error handling, access-control, caching, etc.

use crate::ed25519::{sign_with_ed25519, DerivationPath};
use crate::state::lazy_call_ed25519_public_key;
use crate::state::read_state;
use candid::Principal;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use std::fmt::Display;

#[derive(Clone)]
pub struct SolanaAccount {
    pub ed25519_public_key: Pubkey,
    pub derivation_path: DerivationPath,
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
}

impl SolanaWallet {
    pub async fn new(owner: Principal) -> Self {
        Self { owner }
    }

    pub async fn derive_account(&self, derivation_path: DerivationPath) -> SolanaAccount {
        let extended_key = lazy_call_ed25519_public_key(&derivation_path).await;
        let ed25519_public_key = Pubkey::from(extended_key.public_key);
        SolanaAccount {
            ed25519_public_key,
            derivation_path,
        }
    }

    pub async fn solana_account(&self) -> SolanaAccount {
        self.derive_account(self.owner.as_slice().into()).await
    }

    pub async fn derived_nonce_account(&self) -> SolanaAccount {
        self.derive_account(
            [&self.owner.as_slice(), "nonce-account".as_bytes()]
                .concat()
                .as_slice()
                .into(),
        ).await
    }

    pub async fn sign_with_ed25519(&self, message: &Message, signer: &SolanaAccount) -> Signature {
        let message = message.serialize();
        let derivation_path = signer.derivation_path.clone().into();
        let key_id = read_state(|s| s.ed25519_key_name());
        Signature::from(sign_with_ed25519(message, derivation_path, key_id).await)
    }
}
