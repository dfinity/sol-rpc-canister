use crate::{ed25519::Ed25519ExtendedPublicKey, Ed25519KeyName, InitArg, SolanaNetwork};
use candid::Principal;
use ic_management_canister_types::{
    BoundedVec, SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgs,
    SchnorrPublicKeyResponse,
};
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

thread_local! {
    pub static STATE: RefCell<State> = RefCell::default();
}

pub fn init_state(init_arg: InitArg) {
    STATE.with(|s| *s.borrow_mut() = State::from(init_arg));
}

pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|s| f(s.borrow().deref()))
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|s| f(s.borrow_mut().deref_mut()))
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct State {
    solana_network: SolanaNetwork,
    ed25519_public_key: Option<Ed25519ExtendedPublicKey>,
    ed25519_key_name: Ed25519KeyName,
}

impl State {
    pub fn ed25519_key_id(&self) -> SchnorrKeyId {
        SchnorrKeyId {
            algorithm: SchnorrAlgorithm::Ed25519,
            name: self.ed25519_key_name.to_string(),
        }
    }
}

impl From<InitArg> for State {
    fn from(init_arg: InitArg) -> Self {
        State {
            solana_network: init_arg.solana_network.unwrap_or_default(),
            ed25519_key_name: init_arg.ed5519_key_name.unwrap_or_default(),
            ..Default::default()
        }
    }
}

pub async fn lazy_call_ed25519_public_key() -> Ed25519ExtendedPublicKey {
    if let Some(ed25519_pk) = read_state(|s| s.ed25519_public_key.clone()) {
        return ed25519_pk;
    }
    let key_id = read_state(|s| s.ed25519_key_id());

    let (response,): (SchnorrPublicKeyResponse,) = ic_cdk::call(
        Principal::management_canister(),
        "schnorr_public_key",
        (SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: BoundedVec::new(vec![]),
            key_id,
        },),
    )
    .await
    .unwrap_or_else(|(error_code, message)| {
        ic_cdk::trap(&format!(
            "failed to get canister's public key: {} (error code = {:?})",
            message, error_code,
        ))
    });
    let pk = Ed25519ExtendedPublicKey::from(response);
    mutate_state(|s| s.ed25519_public_key = Some(pk.clone()));
    pk
}
