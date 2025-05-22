//! This module provides some helper functions for the Internet Computer threshold EdDSA signature API in the context
//! of the SOL RPC canister, e.g. signing Solana transactions and fetching and deriving EdDSA public keys.
//! See the [documentation](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-schnorr)
//! for more detailed information on the full threshold Schnorr API.

pub use crate::request::{Request, RequestBuilder, SolRpcEndpoint, SolRpcRequest};
use crate::Runtime;
use candid::Principal;
use derive_more::{From, Into};
use ic_cdk::api::management_canister::schnorr::{
    SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgument, SchnorrPublicKeyResponse,
    SignWithSchnorrArgument, SignWithSchnorrResponse,
};
use sol_rpc_types::{RpcError, RpcResult};
use std::fmt::Display;

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
    TestKeyLocalDevelopment,
    /// Test key available on the ICP mainnet.
    TestKey1,
    /// Production key available on the ICP mainnet.
    ProductionKey1,
}

impl Display for Ed25519KeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Ed25519KeyId::TestKeyLocalDevelopment => "dfx_test_key",
            Ed25519KeyId::TestKey1 => "test_key_1",
            Ed25519KeyId::ProductionKey1 => "key_1",
        }
        .to_string();
        write!(f, "{}", str)
    }
}

/// Sign an unsigned Solana transaction with threshold EdDSA, see threshold Schnorr documentation
/// [here](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-schnorr).
///
/// # Examples
///
/// ```rust
/// use solana_hash::Hash;
/// use solana_message::legacy::Message;
/// use solana_program::system_instruction::transfer;
/// use solana_pubkey::pubkey;
/// use solana_signature::Signature;
/// use solana_transaction::Transaction;
/// use sol_rpc_client::{threshold_sig , IcRuntime};
///
/// #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use sol_rpc_client::fixtures::MockRuntime;
/// # use std::str::FromStr;
/// use candid::Principal;
/// # use ic_cdk::api::management_canister::schnorr::{SchnorrPublicKeyResponse, SignWithSchnorrResponse};
/// let runtime = IcRuntime;
/// # let runtime = MockRuntime::same_response(SchnorrPublicKeyResponse {
/// #     public_key: pubkey!("BPebStjcgCPnWTK3FXZJ8KhqwNYLk9aubC9b4Cgqb6oE").as_ref().to_vec(),
/// #     chain_code: "UWbC6EgDnWEJIU4KFBqASTCYAzEiJGsR".as_bytes().to_vec(),
/// # });
///
/// let key_id = threshold_sig::Ed25519KeyId::TestKey1;
/// let derivation_path = None;
/// let (payer, _) = threshold_sig::get_pubkey(
///     &runtime,
///     None,
///     derivation_path,
///     key_id)
/// .await
/// .unwrap();
///
/// let recipient = pubkey!("BPebStjcgCPnWTK3FXZJ8KhqwNYLk9aubC9b4Cgqb6oE");
///
/// // TODO XC-317: Use client method to fetch recent blockhash
/// let recent_blockhash = Hash::new_unique();
///
/// let message = Message::new_with_blockhash(
///     &[transfer(&payer, &recipient, 1_000_000)],
///     Some(&payer),
///     &recent_blockhash,
///  );
///
///
/// # let runtime = MockRuntime::same_response(SignWithSchnorrResponse {
/// #     signature: Signature::from_str("37HbmunhjSC1xxnVsaFX2xaS8gYnb5JYiLy9B51Ky9Up69aF7Qra6dHSLMCaiurRYq3Y8ZxSVUwC5sntziWuhZee").unwrap().as_ref().to_vec(),
/// # });
/// let mut transaction = Transaction::new_unsigned(message);
/// let signature = threshold_sig::sign_transaction(
///     &runtime,
///     &transaction,
///     key_id,
///     derivation_path,
/// ).await;
///
/// assert_eq!(
///     signature,
///     Ok(Signature::from_str("37HbmunhjSC1xxnVsaFX2xaS8gYnb5JYiLy9B51Ky9Up69aF7Qra6dHSLMCaiurRYq3Y8ZxSVUwC5sntziWuhZee").unwrap())
/// );
///
/// // The transaction is now signed and can be submitted with the `sendTransaction` RPC method.
/// transaction.signatures = vec![signature.unwrap()];
/// # Ok(())
/// # }
/// ```
pub async fn sign_transaction<R: Runtime>(
    runtime: &R,
    transaction: &solana_transaction::Transaction,
    key_id: Ed25519KeyId,
    derivation_path: Option<&DerivationPath>,
) -> RpcResult<solana_signature::Signature> {
    let arg = SignWithSchnorrArgument {
        message: transaction.message_data(),
        derivation_path: derivation_path.cloned().unwrap_or_default().into(),
        key_id: SchnorrKeyId {
            algorithm: SchnorrAlgorithm::Ed25519,
            name: key_id.to_string(),
        },
    };
    let response: SignWithSchnorrResponse = R::update_call(
        runtime,
        Principal::management_canister(),
        "sign_with_schnorr",
        (arg,),
        match key_id {
            Ed25519KeyId::TestKeyLocalDevelopment | Ed25519KeyId::TestKey1 => SIGN_WITH_SCHNORR_TEST_FEE,
            Ed25519KeyId::ProductionKey1 => SIGN_WITH_SCHNORR_PRODUCTION_FEE,
        },
    )
        .await
        .map_err(|(rejection_code, message)| {
            RpcError::ValidationError(format!(
                "Failed to sign transaction, management canister returned code {rejection_code:?}: {message}")
            )
        })?;
    solana_signature::Signature::try_from(response.signature).map_err(|bytes| {
        RpcError::ValidationError(format!(
            "Expected signature to contain 64 bytes, got {} bytes",
            bytes.len()
        ))
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
/// use sol_rpc_client::{threshold_sig, IcRuntime};
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
/// let key_id = threshold_sig::Ed25519KeyId::TestKey1;
/// let canister_id = Principal::from_text("un4fu-tqaaa-aaaab-qadjq-cai").unwrap();
/// let derivation_path = threshold_sig::DerivationPath::from("some-derivation-path".as_bytes());
///
/// let (pubkey, _) = threshold_sig::get_pubkey(
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
) -> RpcResult<(solana_pubkey::Pubkey, [u8; 32])> {
    let arg = SchnorrPublicKeyArgument {
        canister_id,
        derivation_path: derivation_path.cloned().unwrap_or_default().into(),
        key_id: SchnorrKeyId {
            algorithm: SchnorrAlgorithm::Ed25519,
            name: key_id.to_string(),
        },
    };
    let SchnorrPublicKeyResponse {
        public_key, chain_code
    } = runtime
        .query_call(
            Principal::management_canister(),
            "schnorr_public_key",
            (arg,),
        )
        .await
        .map_err(|(rejection_code, message)| {
            RpcError::ValidationError(format!(
                "Failed to fetch EdDSA public key, management canister returned code {rejection_code:?}: {message}")
            )
        })?;
    let pubkey = solana_pubkey::Pubkey::try_from(public_key.as_slice()).map_err(|e| {
        RpcError::ValidationError(format!("Failed to parse bytes as public key: {e}"))
    })?;
    let chain_code = <[u8; 32]>::try_from(chain_code.as_slice()).map_err(|_| {
        RpcError::ValidationError(format!(
            "Expected chain code to contain 32 bytes but it contained {}",
            chain_code.len()
        ))
    })?;
    Ok((pubkey, chain_code))
}
