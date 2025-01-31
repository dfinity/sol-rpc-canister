use ic_crypto_ed25519::{DerivationIndex, PublicKey};
use ic_management_canister_types::{BoundedVec, SchnorrPublicKeyResponse};
use serde_bytes::ByteBuf;

#[derive(Clone)]
pub struct DerivationPath(Vec<Vec<u8>>);

impl From<DerivationPath> for ic_crypto_ed25519::DerivationPath {
    fn from(derivation_path: DerivationPath) -> Self {
        ic_crypto_ed25519::DerivationPath::new(
            derivation_path.0.into_iter().map(DerivationIndex).collect(),
        )
    }
}

impl From<DerivationPath> for ic_management_canister_types::DerivationPath {
    fn from(derivation_path: DerivationPath) -> Self {
        BoundedVec::new(derivation_path.0.into_iter().map(ByteBuf::from).collect())
    }
}

impl From<&[u8]> for DerivationPath {
    fn from(bytes: &[u8]) -> Self {
        const SCHEMA_V1: u8 = 1;
        Self([vec![SCHEMA_V1], bytes.to_vec()].into_iter().collect())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ed25519ExtendedPublicKey {
    pub public_key: PublicKey,
    pub chain_code: [u8; 32],
}

impl Ed25519ExtendedPublicKey {
    pub fn derive_public_key(&self, derivation_path: &DerivationPath) -> Ed25519ExtendedPublicKey {
        let (public_key, chain_code) = self
            .public_key
            .derive_subkey_with_chain_code(&derivation_path.clone().into(), &self.chain_code);
        Self {
            public_key,
            chain_code,
        }
    }
}

impl AsRef<PublicKey> for Ed25519ExtendedPublicKey {
    fn as_ref(&self) -> &PublicKey {
        &self.public_key
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
