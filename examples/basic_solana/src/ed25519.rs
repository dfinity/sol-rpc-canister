use crate::Ed25519KeyName;
use ic_cdk::api::management_canister::schnorr::{
    SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgument, SchnorrPublicKeyResponse,
    SignWithSchnorrArgument, SignWithSchnorrResponse,
};
use ic_crypto_ed25519::PublicKey;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DerivationPath(Vec<Vec<u8>>);

impl From<&[u8]> for DerivationPath {
    fn from(bytes: &[u8]) -> Self {
        const SCHEMA_V1: u8 = 1;
        Self([vec![SCHEMA_V1], bytes.to_vec()].into_iter().collect())
    }
}

impl From<&DerivationPath> for Vec<Vec<u8>> {
    fn from(derivation_path: &DerivationPath) -> Self {
        derivation_path.0.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ed25519ExtendedPublicKey {
    pub public_key: PublicKey,
    pub chain_code: [u8; 32],
}

impl Ed25519ExtendedPublicKey {
    pub fn derive_public_key(&self, derivation_path: &DerivationPath) -> Ed25519ExtendedPublicKey {
        let derivation_path = ic_crypto_ed25519::DerivationPath::new(
            <Vec<Vec<u8>>>::from(derivation_path)
                .into_iter()
                .map(ic_crypto_ed25519::DerivationIndex)
                .collect(),
        );
        let (public_key, chain_code) = self
            .public_key
            .derive_subkey_with_chain_code(&derivation_path, &self.chain_code);
        Self {
            public_key,
            chain_code,
        }
    }
}

impl From<SchnorrPublicKeyResponse> for Ed25519ExtendedPublicKey {
    fn from(value: SchnorrPublicKeyResponse) -> Self {
        Ed25519ExtendedPublicKey {
            public_key: PublicKey::deserialize_raw(value.public_key.as_slice()).unwrap(),
            chain_code: <[u8; 32]>::try_from(value.chain_code).unwrap(),
        }
    }
}

pub async fn get_ed25519_public_key(
    key_name: &Ed25519KeyName,
    derivation_path: &DerivationPath,
) -> Ed25519ExtendedPublicKey {
    let (response,): (SchnorrPublicKeyResponse,) =
        ic_cdk::api::management_canister::schnorr::schnorr_public_key(SchnorrPublicKeyArgument {
            canister_id: None,
            derivation_path: derivation_path.into(),
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: key_name.to_string(),
            },
        })
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
    derivation_path: &DerivationPath,
    key_name: &Ed25519KeyName,
) -> [u8; 64] {
    let (response,): (SignWithSchnorrResponse,) =
        ic_cdk::api::management_canister::schnorr::sign_with_schnorr(SignWithSchnorrArgument {
            message,
            derivation_path: derivation_path.into(),
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: key_name.to_string(),
            },
        })
        .await
        .expect("failed to sign with ed25519");
    let signature_length = response.signature.len();
    <[u8; 64]>::try_from(response.signature).unwrap_or_else(|_| {
        panic!(
            "BUG: invalid signature from management canister. Expected 64 bytes but got {} bytes",
            signature_length
        )
    })
}
