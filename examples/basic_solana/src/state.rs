use std::{cell::RefCell, ops::{Deref, DerefMut}};
use crate::{InitArg, SolanaNetwork};

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
}

impl From<InitArg> for State {
    fn from(init_arg: InitArg) -> Self {
        State {
            solana_network: init_arg.solana_network.unwrap_or_default(),
        }
    }
}