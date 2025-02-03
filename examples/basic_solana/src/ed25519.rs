use candid::Principal;
use sol_rpc_types::management_canister::{
    SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgs, SchnorrPublicKeyResponse,
    SignWithSchnorrArgs, SignWithSchnorrResponse,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd)]
pub struct DerivationPath(Vec<Vec<u8>>);

#[derive(Debug, PartialEq, Eq, Clone, Ord, PartialOrd, Copy)]
pub struct Ed25519ExtendedPublicKey {
    pub public_key: [u8; 32],
    pub chain_code: [u8; 32],
}

impl From<&[u8]> for DerivationPath {
    fn from(bytes: &[u8]) -> Self {
        const SCHEMA_V1: u8 = 1;
        Self([vec![SCHEMA_V1], bytes.to_vec()].into_iter().collect())
    }
}

impl From<DerivationPath> for Vec<Vec<u8>> {
    fn from(derivation_path: DerivationPath) -> Self {
        derivation_path.0
    }
}

impl From<SchnorrPublicKeyResponse> for Ed25519ExtendedPublicKey {
    fn from(value: SchnorrPublicKeyResponse) -> Self {
        Ed25519ExtendedPublicKey {
            public_key: <[u8; 32]>::try_from(value.public_key).unwrap(),
            chain_code: <[u8; 32]>::try_from(value.chain_code).unwrap(),
        }
    }
}

pub async fn get_ed25519_public_key(
    key_name: String,
    derivation_path: &DerivationPath,
) -> Ed25519ExtendedPublicKey {
    let (response,): (SchnorrPublicKeyResponse,) = ic_cdk::call(
        Principal::management_canister(),
        "schnorr_public_key",
        (SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: derivation_path.clone().into(),
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: key_name,
            },
        },),
    )
    .await
    .unwrap_or_else(|(error_code, message)| {
        ic_cdk::trap(&format!(
            "failed to get canister's public key: {} (error code = {:?})",
            message, error_code,
        ))
    });
    Ed25519ExtendedPublicKey::from(response)
}

pub async fn sign_with_ed25519(
    message: Vec<u8>,
    derivation_path: DerivationPath,
    key_name: String,
) -> [u8; 64] {
    let (response,): (SignWithSchnorrResponse,) = ic_cdk::call(
        Principal::management_canister(),
        "sign_with_schnorr",
        (SignWithSchnorrArgs {
            message,
            derivation_path: derivation_path.into(),
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: key_name,
            },
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
    signature
}
