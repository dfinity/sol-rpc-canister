use crate::Ed25519KeyName;
use ic_ed25519::PublicKey;
use sol_rpc_client::{ed25519::DerivationPath, IcRuntime};

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

pub async fn get_ed25519_public_key(
    key_name: Ed25519KeyName,
    derivation_path: &DerivationPath,
) -> Ed25519ExtendedPublicKey {
    let (pubkey, chain_code) = sol_rpc_client::ed25519::get_pubkey(
        &IcRuntime,
        None,
        Some(derivation_path),
        key_name.into(),
    )
    .await
    .expect("Failed to fetch EdDSA public key");
    Ed25519ExtendedPublicKey {
        public_key: PublicKey::deserialize_raw(&pubkey.to_bytes()).unwrap(),
        chain_code,
    }
}
