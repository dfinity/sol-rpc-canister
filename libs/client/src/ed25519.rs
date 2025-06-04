//! This module provides some helper functions for the Internet Computer threshold EdDSA signature API in the context
//! of the SOL RPC canister, e.g. signing Solana transactions and fetching and deriving EdDSA public keys.
//! See the [documentation](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-schnorr)
//! for more detailed information on the full threshold Schnorr API.

use crate::Runtime;
use candid::Principal;
use derive_more::{From, Into};
use ic_cdk::api::{
    call::RejectionCode,
    management_canister::schnorr::{
        SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgument, SchnorrPublicKeyResponse,
        SignWithSchnorrArgument, SignWithSchnorrResponse,
    },
};

// Source: https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-schnorr-test-key
const SIGN_WITH_SCHNORR_TEST_FEE: u128 = 10_000_000_000;
// Source: https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-schnorr-production-key
const SIGN_WITH_SCHNORR_PRODUCTION_FEE: u128 = 26_153_846_153;

/// Represents the derivation path of an Ed25519 key from one of the root keys.
/// See the [tEdDSA documentation](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-schnorr#signing-messages-and-transactions)
/// for more details.
#[derive(Clone, Debug, PartialEq, Eq, Default, From, Into)]
pub struct DerivationPath(Vec<Vec<u8>>);

impl From<&[&[u8]]> for DerivationPath {
    fn from(bytes: &[&[u8]]) -> Self {
        Self(bytes.iter().map(|index| index.to_vec()).collect())
    }
}

impl From<&[u8]> for DerivationPath {
    fn from(bytes: &[u8]) -> Self {
        Self(vec![bytes.to_vec()])
    }
}

impl From<Principal> for DerivationPath {
    fn from(principal: Principal) -> Self {
        DerivationPath::from(principal.as_slice())
    }
}

/// The ID of one of the ICP root keys.
/// See the [tEdDSA documentation](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-schnorr#signing-messages-and-transactions)
/// for more details.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ed25519KeyId {
    /// Only available on the local development environment started by `dfx`.
    LocalDevelopment,
    /// Test key available on the ICP mainnet.
    MainnetTestKey1,
    /// Production key available on the ICP mainnet.
    MainnetProdKey1,
}

impl Ed25519KeyId {
    /// The string representation of a [`Ed25519KeyId`] used as an argument to threshold Schnorr
    /// method calls such as `schnorr_public_key` or `sign_with_schnorr`.
    pub fn id(&self) -> &'static str {
        match self {
            Ed25519KeyId::LocalDevelopment => "dfx_test_key",
            Ed25519KeyId::MainnetTestKey1 => "test_key_1",
            Ed25519KeyId::MainnetProdKey1 => "key_1",
        }
    }
}

/// Sign a Solana message with threshold EdDSA, see threshold Schnorr documentation
/// [here](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-schnorr).
///
/// # Examples
///
/// ```rust
/// use candid::Principal;
/// use solana_hash::Hash;
/// use solana_message::legacy::Message;
/// use solana_program::system_instruction::transfer;
/// use solana_pubkey::pubkey;
/// use solana_signature::Signature;
/// use solana_transaction::Transaction;
/// use sol_rpc_client::{
///     ed25519::{get_pubkey, sign_message, DerivationPath, Ed25519KeyId},
///     IcRuntime, SolRpcClient
/// };
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use sol_rpc_client::fixtures::MockRuntime;
/// # use std::str::FromStr;
/// # use ic_cdk::api::management_canister::schnorr::{SchnorrPublicKeyResponse, SignWithSchnorrResponse};
/// let runtime = IcRuntime;
/// # let runtime = MockRuntime::same_response(SchnorrPublicKeyResponse {
/// #     public_key: pubkey!("BPebStjcgCPnWTK3FXZJ8KhqwNYLk9aubC9b4Cgqb6oE").as_ref().to_vec(),
/// #     chain_code: "UWbC6EgDnWEJIU4KFBqASTCYAzEiJGsR".as_bytes().to_vec(),
/// # });
///
/// let key_id = Ed25519KeyId::MainnetTestKey1;
/// let derivation_path = DerivationPath::from(
///     Principal::from_text("vaupb-eqaaa-aaaai-qplka-cai").unwrap()
/// );
/// let (payer, _) = get_pubkey(
///     &runtime,
///     None,
///     Some(&derivation_path),
///     key_id
/// )
/// .await
/// .unwrap();
///
/// let recipient = pubkey!("BPebStjcgCPnWTK3FXZJ8KhqwNYLk9aubC9b4Cgqb6oE");
///
/// # use sol_rpc_types::MultiRpcResult;
/// let blockhash = SolRpcClient::builder_for_ic()
/// #   .with_mocked_responses(
/// #        MultiRpcResult::Consistent(Ok(332_577_897_u64)),
/// #        MultiRpcResult::Consistent(Ok(332_577_897_u64)),
/// #    )
///     .build()
///     .estimate_recent_blockhash()
///     .send()
///     .await
///     .expect("Failed to fetch recent blockhash");
///
/// let message = Message::new_with_blockhash(
///     &[transfer(&payer, &recipient, 1_000_000)],
///     Some(&payer),
///     &blockhash,
///  );
///
/// # let runtime = MockRuntime::same_response(SignWithSchnorrResponse {
/// #     signature: Signature::from_str("37HbmunhjSC1xxnVsaFX2xaS8gYnb5JYiLy9B51Ky9Up69aF7Qra6dHSLMCaiurRYq3Y8ZxSVUwC5sntziWuhZee").unwrap().as_ref().to_vec(),
/// # });
/// let signature = sign_message(
///     &runtime,
///     &message,
///     key_id,
///     Some(&derivation_path),
/// )
/// .await;
///
/// assert_eq!(
///     signature,
///     Ok(Signature::from_str("37HbmunhjSC1xxnVsaFX2xaS8gYnb5JYiLy9B51Ky9Up69aF7Qra6dHSLMCaiurRYq3Y8ZxSVUwC5sntziWuhZee").unwrap())
/// );
///
/// let transaction = Transaction {
///     message,
///     signatures: vec![signature.unwrap()],
/// };
/// # Ok(())
/// # }
/// ```
pub async fn sign_message<R: Runtime>(
    runtime: &R,
    message: &solana_message::Message,
    key_id: Ed25519KeyId,
    derivation_path: Option<&DerivationPath>,
) -> Result<solana_signature::Signature, (RejectionCode, String)> {
    let arg = SignWithSchnorrArgument {
        message: message.serialize(),
        derivation_path: derivation_path.cloned().unwrap_or_default().into(),
        key_id: SchnorrKeyId {
            algorithm: SchnorrAlgorithm::Ed25519,
            name: key_id.id().to_string(),
        },
    };
    let response: SignWithSchnorrResponse = R::update_call(
        runtime,
        Principal::management_canister(),
        "sign_with_schnorr",
        (arg,),
        match key_id {
            Ed25519KeyId::LocalDevelopment | Ed25519KeyId::MainnetTestKey1 => {
                SIGN_WITH_SCHNORR_TEST_FEE
            }
            Ed25519KeyId::MainnetProdKey1 => SIGN_WITH_SCHNORR_PRODUCTION_FEE,
        },
    )
    .await?;
    solana_signature::Signature::try_from(response.signature).map_err(|e| {
        panic!(
            "Expected signature to contain 64 bytes, got {} bytes",
            e.len()
        )
    })
}

/// Fetch the Ed25519 public key for the key ID, given canister ID and derivation path, see threshold Schnorr
/// documentation [here](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-schnorr).
///
/// # Examples
///
/// ```rust
/// use candid::Principal;
/// use solana_pubkey::pubkey;
/// use sol_rpc_client::{
///     ed25519::{get_pubkey, DerivationPath, Ed25519KeyId},
///     IcRuntime
/// };
///
/// #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use sol_rpc_client::fixtures::MockRuntime;
/// # use ic_cdk::api::management_canister::schnorr::{SchnorrPublicKeyResponse, SignWithSchnorrResponse};
/// let runtime = IcRuntime;
/// # let runtime = MockRuntime::same_response(SchnorrPublicKeyResponse {
/// #     public_key: pubkey!("BPebStjcgCPnWTK3FXZJ8KhqwNYLk9aubC9b4Cgqb6oE").as_ref().to_vec(),
/// #     chain_code: "UWbC6EgDnWEJIU4KFBqASTCYAzEiJGsR".as_bytes().to_vec(),
/// # });
///
/// let key_id = Ed25519KeyId::MainnetTestKey1;
/// let canister_id = Principal::from_text("un4fu-tqaaa-aaaab-qadjq-cai").unwrap();
/// let derivation_path = DerivationPath::from(
///     Principal::from_text("vaupb-eqaaa-aaaai-qplka-cai").unwrap()
/// );
/// let (payer, _) = get_pubkey(
///     &runtime,
///     None,
///     Some(&derivation_path),
///     key_id
/// )
/// .await
/// .unwrap();
///
/// let (pubkey, _) = get_pubkey(
///     &runtime,
///     Some(canister_id),
///     Some(&derivation_path),
///     key_id
/// )
/// .await
/// .unwrap();
///
/// assert_eq!(pubkey, pubkey!("BPebStjcgCPnWTK3FXZJ8KhqwNYLk9aubC9b4Cgqb6oE")
/// );
/// # Ok(())
/// # }
/// ```
pub async fn get_pubkey<R: Runtime>(
    runtime: &R,
    canister_id: Option<Principal>,
    derivation_path: Option<&DerivationPath>,
    key_id: Ed25519KeyId,
) -> Result<(solana_pubkey::Pubkey, [u8; 32]), (RejectionCode, String)> {
    let arg = SchnorrPublicKeyArgument {
        canister_id,
        derivation_path: derivation_path.cloned().unwrap_or_default().into(),
        key_id: SchnorrKeyId {
            algorithm: SchnorrAlgorithm::Ed25519,
            name: key_id.id().to_string(),
        },
    };
    let SchnorrPublicKeyResponse {
        public_key,
        chain_code,
    } = runtime
        .update_call(
            Principal::management_canister(),
            "schnorr_public_key",
            (arg,),
            0,
        )
        .await?;
    let pubkey = solana_pubkey::Pubkey::try_from(public_key).unwrap_or_else(|e| {
        panic!(
            "Expected public key to contain 32 bytes, got {} bytes",
            e.len()
        )
    });
    let chain_code = <[u8; 32]>::try_from(chain_code).unwrap_or_else(|e| {
        panic!(
            "Expected chain code key to contain 32 bytes, got {} bytes",
            e.len()
        )
    });
    Ok((pubkey, chain_code))
}
