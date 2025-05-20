use candid::{CandidType, Deserialize, Principal};
use derive_more::{From, Into};
use std::fmt::Display;

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
#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Ed25519KeyId {
    /// Only available on the local development environment started by dfx.
    #[default]
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
