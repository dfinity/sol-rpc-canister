use ic_crypto_ed25519::{DerivationPath, PublicKey};
use ic_management_canister_types::SchnorrPublicKeyResponse;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ed25519ExtendedPublicKey {
    pub public_key: PublicKey,
    pub chain_code: [u8; 32],
}

impl Ed25519ExtendedPublicKey {
    pub fn derive_new_public_key(&self, derivation_path: &DerivationPath) -> Ed25519ExtendedPublicKey {
        let (public_key, chain_code) = self
            .public_key
            .derive_subkey_with_chain_code(derivation_path, &self.chain_code);
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
