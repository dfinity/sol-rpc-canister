#[cfg(test)]
mod tests;

use crate::{
    metrics::Metrics,
    types::{ApiKey, OverrideProvider},
};
use candid::{Deserialize, Principal};
use canhttp::http::json::{ConstantSizeId, Id};
use canlog::LogFilter;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
    Cell, DefaultMemoryImpl, Storable,
};
use serde::Serialize;
use sol_rpc_types::{InstallArgs, Mode, SupportedRpcProviderId};
use std::{borrow::Cow, cell::RefCell, collections::BTreeMap};

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);

type StableMemory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // Unstable static data: these are reset when the canister is upgraded.
    pub static UNSTABLE_METRICS: RefCell<Metrics> = RefCell::new(Metrics::default());
    static UNSTABLE_HTTP_REQUEST_COUNTER: RefCell<ConstantSizeId> = const {RefCell::new(ConstantSizeId::ZERO)};

    // Stable static data: these are preserved when the canister is upgraded.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static STATE: RefCell<Cell<ConfigState, StableMemory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(STATE_MEMORY_ID)),
            ConfigState::default(),
        )
        .expect("Unable to read memory from stable memory"),
    );
}

/// Configuration memory of the ledger orchestrator.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum ConfigState {
    // This memory is only used between wasm module initialization and init().
    #[default]
    Uninitialized,
    Initialized(State),
}

impl ConfigState {
    fn expect_initialized(&self) -> &State {
        match &self {
            ConfigState::Uninitialized => ic_cdk::trap("BUG: memory not initialized"),
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
    ciborium::ser::into_writer(state, &mut buf).expect("failed to encode memory");
    buf
}

fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> T {
    ciborium::de::from_reader(bytes)
        .unwrap_or_else(|e| panic!("failed to decode memory bytes {}: {e}", hex::encode(bytes)))
}

#[derive(Default, Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct State {
    api_keys: BTreeMap<SupportedRpcProviderId, ApiKey>,
    api_key_principals: Vec<Principal>,
    override_provider: OverrideProvider,
    log_filter: LogFilter,
    mode: Mode,
    num_subnet_nodes: u32,
}

impl State {
    pub fn get_api_key(&self, provider: &SupportedRpcProviderId) -> Option<ApiKey> {
        self.api_keys.get(provider).cloned()
    }

    pub fn insert_api_key(&mut self, provider: SupportedRpcProviderId, api_key: ApiKey) {
        self.api_keys.insert(provider, api_key);
    }

    pub fn remove_api_key(&mut self, provider: &SupportedRpcProviderId) {
        self.api_keys.remove(provider);
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

    pub fn get_num_subnet_nodes(&self) -> u32 {
        self.num_subnet_nodes
    }

    pub fn set_num_subnet_nodes(&mut self, num_subnet_nodes: u32) {
        self.num_subnet_nodes = num_subnet_nodes
    }

    pub fn get_mode(&self) -> Mode {
        self.mode
    }

    pub fn is_demo_mode_active(&self) -> bool {
        self.mode == Mode::Demo
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode
    }
}

impl From<InstallArgs> for State {
    fn from(value: InstallArgs) -> Self {
        Self {
            api_keys: Default::default(),
            api_key_principals: value.manage_api_keys.unwrap_or_default(),
            override_provider: value.override_provider.unwrap_or_default().into(),
            log_filter: value.log_filter.unwrap_or_default(),
            mode: value.mode.unwrap_or_default(),
            num_subnet_nodes: value.num_subnet_nodes.unwrap_or_default().into(),
        }
    }
}

pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|cell| f(cell.borrow().get().expect_initialized()))
}

/// Mutates (part of) the current memory using `f`.
///
/// Panics if there is no memory.
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
            .expect("failed to write memory in stable cell");
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
            .expect("failed to initialize memory in stable cell")
    });
}

/// Resets the memory to [`ConfigState::Uninitialized`] which is useful e.g. in property tests where
/// the thread gets re-used and thus the memory persists across test instances.
pub fn reset_state() {
    STATE.with(|cell| {
        cell.borrow_mut()
            .set(ConfigState::Uninitialized)
            .unwrap_or_else(|err| panic!("Could not reset memory: {:?}", err));
    })
}

pub fn next_request_id() -> Id {
    UNSTABLE_HTTP_REQUEST_COUNTER.with_borrow_mut(|counter| {
        let current_request_id = counter.get_and_increment();
        Id::from(current_request_id)
    })
}
