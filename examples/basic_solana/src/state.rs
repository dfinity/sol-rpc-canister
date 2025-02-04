use crate::{
    ed25519::{get_ed25519_public_key, Ed25519ExtendedPublicKey},
    Ed25519KeyName, InitArg, SolanaNetwork,
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
    pub fn ed25519_key_name(&self) -> Ed25519KeyName {
        self.ed25519_key_name.clone()
    }

    pub fn solana_network(&self) -> SolanaNetwork {
        self.solana_network
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
    if let Some(public_key) = read_state(|s| s.ed25519_public_key.clone()) {
        return public_key;
    }
    let public_key =
        get_ed25519_public_key(&read_state(|s| s.ed25519_key_name()), &Default::default()).await;
    mutate_state(|s| s.ed25519_public_key = Some(public_key.clone()));
    public_key
}
