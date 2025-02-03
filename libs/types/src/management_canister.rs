use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use serde_bytes::ByteBuf;

pub type CanisterId = Principal;

/// Types of algorithms that can be used for Schnorr signing.
#[derive(CandidType, Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum SchnorrAlgorithm {
    #[serde(rename = "bip340secp256k1")]
    Bip340Secp256k1,
    #[serde(rename = "ed25519")]
    Ed25519,
}

/// Unique identifier for a key that can be used for Schnorr signatures. The name
/// is just a identifier, but it may be used to convey some information about
/// the key (e.g. that the key is meant to be used for testing purposes).
#[derive(CandidType, Serialize, Debug, Clone)]
pub struct SchnorrKeyId {
    pub algorithm: SchnorrAlgorithm,
    pub name: String,
}

/// Represents the argument of the schnorr_public_key API.
#[derive(CandidType, Serialize, Debug)]
pub struct SchnorrPublicKeyArgs {
    pub canister_id: Option<CanisterId>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,
}

/// Represents the response of the schnorr_public_key API.
#[derive(CandidType, Deserialize, Debug)]
pub struct SchnorrPublicKeyResponse {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

/// Represents the argument of the sign_with_schnorr API.
#[derive(CandidType, Serialize, Debug)]
pub struct SignWithSchnorrArgs {
    pub message: Vec<u8>,
    pub aux: Option<SignWithSchnorrAux>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,
}

/// Represents the response of the sign_with_schnorr API.
#[derive(CandidType, Deserialize, Debug)]
pub struct SignWithSchnorrResponse {
    pub signature: Vec<u8>,
}

/// Represents the aux argument of the sign_with_schnorr API.
#[derive(Eq, PartialEq, Debug, CandidType, Serialize)]
pub enum SignWithSchnorrAux {
    #[serde(rename = "bip341")]
    Bip341(SignWithBip341Aux),
}

/// Represents the BIP341 aux argument of the sign_with_schnorr API.
#[derive(Eq, PartialEq, Debug, CandidType, Serialize)]
pub struct SignWithBip341Aux {
    pub merkle_root_hash: ByteBuf,
}
