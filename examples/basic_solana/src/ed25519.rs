use ic_cdk::api::management_canister::schnorr::{
    SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgument, SchnorrPublicKeyResponse,
};
use ic_ed25519::PublicKey;
use sol_rpc_types::{DerivationPath, Ed25519KeyId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ed25519ExtendedPublicKey {
    pub public_key: PublicKey,
    pub chain_code: [u8; 32],
}

impl Ed25519ExtendedPublicKey {
    pub fn derive_public_key(&self, derivation_path: DerivationPath) -> Ed25519ExtendedPublicKey {
        let derivation_path = ic_ed25519::DerivationPath::new(
            <Vec<Vec<u8>>>::from(derivation_path)
                .into_iter()
                .map(ic_ed25519::DerivationIndex)
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
    key_name: &Ed25519KeyId,
    derivation_path: &DerivationPath,
) -> Ed25519ExtendedPublicKey {
    let (response,): (SchnorrPublicKeyResponse,) =
        ic_cdk::api::management_canister::schnorr::schnorr_public_key(SchnorrPublicKeyArgument {
            canister_id: None,
            derivation_path: derivation_path.clone().into(),
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
