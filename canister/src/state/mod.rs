#[cfg(test)]
mod tests;

use crate::types::{ApiKey, OverrideProvider};
use candid::{Deserialize, Principal};
use canlog::LogFilter;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
    Cell, DefaultMemoryImpl, Storable,
};
use serde::Serialize;
use sol_rpc_types::{InstallArgs, ProviderId, SolanaCluster};
use std::{borrow::Cow, cell::RefCell, collections::BTreeMap};

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);

type StableMemory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // Unstable static data: these are reset when the canister is upgraded.
    // TODO: Add metrics

    // Stable static data: these are preserved when the canister is upgraded.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static STATE: RefCell<Cell<ConfigState, StableMemory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(STATE_MEMORY_ID)),
            ConfigState::default(),
        )
        .expect("Unable to read state from stable memory"),
    );
}

/// Configuration state of the ledger orchestrator.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum ConfigState {
    // This state is only used between wasm module initialization and init().
    #[default]
    Uninitialized,
    Initialized(State),
}

impl ConfigState {
    fn expect_initialized(&self) -> &State {
        match &self {
            ConfigState::Uninitialized => ic_cdk::trap("BUG: state not initialized"),
            ConfigState::Initialized(s) => s,
        }
    }
}

impl Storable for ConfigState {
    fn to_bytes(&self) -> Cow<[u8]> {
        match &self {
            ConfigState::Uninitialized => Cow::Borrowed(&[]),
            ConfigState::Initialized(config) => Cow::Owned(encode(config)),
        }
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        if bytes.is_empty() {
            return ConfigState::Uninitialized;
        }
        ConfigState::Initialized(decode(bytes.as_ref()))
    }

    const BOUND: Bound = Bound::Unbounded;
}

fn encode<S: ?Sized + serde::Serialize>(state: &S) -> Vec<u8> {
    let mut buf = vec![];
    ciborium::ser::into_writer(state, &mut buf).expect("failed to encode state");
    buf
}

fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> T {
    ciborium::de::from_reader(bytes)
        .unwrap_or_else(|e| panic!("failed to decode state bytes {}: {e}", hex::encode(bytes)))
}

#[derive(Default, Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct State {
    api_keys: BTreeMap<(ProviderId, SolanaCluster), ApiKey>,
    api_key_principals: Vec<Principal>,
    override_provider: OverrideProvider,
    log_filter: LogFilter,
}

impl State {
    pub fn get_api_key(&self, (provider, cluster): (ProviderId, SolanaCluster)) -> Option<ApiKey> {
        self.api_keys.get(&(provider, cluster)).cloned()
    }

    pub fn insert_api_key(&mut self, (provider, cluster): (ProviderId, SolanaCluster), api_key: ApiKey) {
        self.api_keys.insert((provider, cluster), api_key);
    }

    pub fn remove_api_key(&mut self, (provider, cluster): (ProviderId, SolanaCluster)) {
        self.api_keys.remove(&(provider, cluster));
    }

    pub fn is_api_key_principal(&self, principal: &Principal) -> bool {
        self.api_key_principals
            .iter()
            .any(|other| other == principal)
    }

    pub fn set_api_key_principals(&mut self, new_principals: Vec<Principal>) {
        while !self.api_key_principals.is_empty() {
            self.api_key_principals.pop();
        }
        for principal in new_principals {
            self.api_key_principals.push(principal);
        }
    }

    pub fn get_override_provider(&self) -> OverrideProvider {
        self.override_provider.clone()
    }

    pub fn set_override_provider(&mut self, override_provider: OverrideProvider) {
        self.override_provider = override_provider
    }

    pub fn get_log_filter(&self) -> LogFilter {
        self.log_filter.clone()
    }

    pub fn set_log_filter(&mut self, filter: LogFilter) {
        self.log_filter = filter;
    }
}

impl From<InstallArgs> for State {
    fn from(value: InstallArgs) -> Self {
        Self {
            api_keys: Default::default(),
            api_key_principals: value.manage_api_keys.unwrap_or_default(),
            override_provider: value.override_provider.unwrap_or_default().into(),
            log_filter: value.log_filter.unwrap_or_default(),
        }
    }
}

pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|cell| f(cell.borrow().get().expect_initialized()))
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|cell| {
        let mut borrowed = cell.borrow_mut();
        let mut state = borrowed.get().expect_initialized().clone();
        let result = f(&mut state);
        borrowed
            .set(ConfigState::Initialized(state))
            .expect("failed to write state in stable cell");
        result
    })
}

pub fn init_state(state: State) {
    STATE.with(|cell| {
        let mut borrowed = cell.borrow_mut();
        assert_eq!(
            borrowed.get(),
            &ConfigState::Uninitialized,
            "BUG: State is already initialized and has value {:?}",
            borrowed.get()
        );
        borrowed
            .set(ConfigState::Initialized(state))
            .expect("failed to initialize state in stable cell")
    });
}

/// Resets the state to [`ConfigState::Uninitialized`] which is useful e.g. in property tests where
/// the thread gets re-used and thus the state persists across test instances.
pub fn reset_state() {
    STATE.with(|cell| {
        cell.borrow_mut()
            .set(ConfigState::Uninitialized)
            .unwrap_or_else(|err| panic!("Could not reset state: {:?}", err));
    })
}
