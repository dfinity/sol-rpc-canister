use crate::{
    ed25519::{get_ed25519_public_key, Ed25519ExtendedPublicKey},
    ed25519_key_id, InitArg, SolanaNetwork,
};
use sol_rpc_types::CommitmentLevel;
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

#[derive(Debug, PartialEq, Eq)]
pub struct State {
    solana_network: SolanaNetwork,
    solana_commitment_level: CommitmentLevel,
    ed25519_public_key: Option<Ed25519ExtendedPublicKey>,
    ed25519_key_name: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            solana_network: SolanaNetwork::default(),
            solana_commitment_level: CommitmentLevel::default(),
            ed25519_public_key: None,
            ed25519_key_name: "test_key_1".to_string(),
        }
    }
}

impl State {
    pub fn ed25519_key_name(&self) -> &str {
        &self.ed25519_key_name
    }

    pub fn solana_network(&self) -> &SolanaNetwork {
        &self.solana_network
    }

    pub fn solana_commitment_level(&self) -> CommitmentLevel {
        self.solana_commitment_level.clone()
    }
}

impl From<InitArg> for State {
    fn from(init_arg: InitArg) -> Self {
        let ed25519_key_name = init_arg
            .ed25519_key_name
            .unwrap_or_else(|| "test_key_1".to_string());
        // Validate the key name eagerly so an unsupported value fails at install time
        // rather than on the first signing request.
        let _ = ed25519_key_id(&ed25519_key_name);
        State {
            solana_network: init_arg.solana_network.unwrap_or_default(),
            solana_commitment_level: init_arg.solana_commitment_level.unwrap_or_default(),
            ed25519_public_key: None,
            ed25519_key_name,
        }
    }
}

pub async fn lazy_call_ed25519_public_key() -> Ed25519ExtendedPublicKey {
    if let Some(public_key) = read_state(|s| s.ed25519_public_key.clone()) {
        return public_key;
    }
    let key_id = read_state(|s| ed25519_key_id(s.ed25519_key_name()));
    let public_key = get_ed25519_public_key(key_id, &Default::default()).await;
    mutate_state(|s| s.ed25519_public_key = Some(public_key.clone()));
    public_key
}
