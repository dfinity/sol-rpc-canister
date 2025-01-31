//! A demo of a very bare-bones Solana "wallet".
//!
//! The wallet here showcases how Solana addresses can be computed and how Solana transactions
//! can be signed. It is missing several pieces that any production-grade wallet would have,
//! such as error handling, access-control, caching, etc.

use crate::ed25519::DerivationPath;
use crate::state::read_state;
use crate::{ed25519::Ed25519ExtendedPublicKey, state::lazy_call_ed25519_public_key};
use candid::Principal;
use ic_cdk::api::management_canister::schnorr::SignWithSchnorrResponse;
use ic_management_canister_types::SignWithSchnorrArgs;
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
        let ed25519_public_key = self
            .root_public_key
            .derive_public_key(&derivation_path)
            .public_key
            .serialize_raw()
            .into();
        SolanaAccount {
            ed25519_public_key,
            derivation_path,
        }
    }

    pub fn solana_account(&self) -> SolanaAccount {
        self.derive_account(self.owner.as_slice().into())
    }

    pub fn derived_nonce_account(&self) -> SolanaAccount {
        self.derive_account(
            [&self.owner.as_slice(), "nonce-account".as_bytes()]
                .concat()
                .as_slice()
                .into(),
        )
    }

    pub async fn sign_with_ed25519(&self, message: &Message, signer: &SolanaAccount) -> Signature {
        let message = message.serialize();
        let derivation_path = signer.derivation_path.clone().into();
        let key_id = read_state(|s| s.ed25519_key_id());

        let (response,): (SignWithSchnorrResponse,) = ic_cdk::call(
            Principal::management_canister(),
            "sign_with_schnorr",
            (SignWithSchnorrArgs {
                message,
                derivation_path,
                key_id,
                aux: None,
            },),
        )
        .await
        .expect("failed to sign with ed25519");
        let signature_length = response.signature.len();
        let signature = <[u8; 64]>::try_from(response.signature).unwrap_or_else(|_| {
            panic!(
                "BUG: invalid signature from management canister. Expected 64 bytes but got {} bytes",
                signature_length
            )
        });
        Signature::from(signature)
    }
}
