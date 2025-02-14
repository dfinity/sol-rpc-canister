use crate::types::{ApiKey, ProviderId};
use candid::Principal;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    Cell, DefaultMemoryImpl, Memory, StableBTreeMap,
};
use sol_rpc_types::ProviderId;
use std::cell::RefCell;
use std::collections::BTreeMap;

pub struct State {
    pub api_keys: BTreeMap<ProviderId, ApiKey>,
    pub api_key_principals: Vec<Principal>,
}

impl Default for State {
    fn new<M>(memory: M) -> Self
    where
        M: Memory,
    {
        Self {
            api_keys: StableBTreeMap::new(memory),
            api_key_principals: ic_stable_structures::Vec::init(memory),
        }
    }
}

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);

type StableMemory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // Stable static data: these are preserved when the canister is upgraded.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
   static STATE: RefCell<Cell<State, StableMemory>> = RefCell::new(Cell::init(MEMORY_MANAGER.with_borrow(|m| m.get(STATE_MEMORY_ID)), State::default()).expect("Unable to read state from stable memory"));
}

pub fn get_api_key(provider_id: ProviderId) -> Option<ApiKey> {
    STATE.with_borrow_mut(|state| state.api_keys.get(&provider_id))
}

pub fn insert_api_key(provider_id: ProviderId, api_key: ApiKey) {
    STATE.with_borrow_mut(|state| state.api_keys.insert(provider_id, api_key));
}

pub fn remove_api_key(provider_id: ProviderId) {
    STATE.with_borrow_mut(|state| state.api_keys.remove(&provider_id));
}

pub fn is_api_key_principal(principal: &Principal) -> bool {
    STATE.with_borrow(|state| {
        state
            .api_key_principals
            .iter()
            .any(|other| &other == principal)
    })
}

pub fn set_api_key_principals(new_principals: Vec<Principal>) {
    STATE.with_borrow_mut(|state| {
        while !state.api_key_principals.is_empty() {
            state.api_key_principals.pop();
        }
        for principal in new_principals {
            state
                .api_key_principals
                .push(&principal)
                .expect("Error while adding API key principal");
        }
    });
}

#[cfg(test)]
mod test {
    use candid::Principal;

    use crate::memory::{is_api_key_principal, set_api_key_principals};

    #[test]
    fn test_api_key_principals() {
        let principal1 =
            Principal::from_text("k5dlc-ijshq-lsyre-qvvpq-2bnxr-pb26c-ag3sc-t6zo5-rdavy-recje-zqe")
                .unwrap();
        let principal2 =
            Principal::from_text("yxhtl-jlpgx-wqnzc-ysego-h6yqe-3zwfo-o3grn-gvuhm-nz3kv-ainub-6ae")
                .unwrap();
        assert!(!is_api_key_principal(&principal1));
        assert!(!is_api_key_principal(&principal2));

        set_api_key_principals(vec![principal1]);
        assert!(is_api_key_principal(&principal1));
        assert!(!is_api_key_principal(&principal2));

        set_api_key_principals(vec![principal2]);
        assert!(!is_api_key_principal(&principal1));
        assert!(is_api_key_principal(&principal2));

        set_api_key_principals(vec![principal1, principal2]);
        assert!(is_api_key_principal(&principal1));
        assert!(is_api_key_principal(&principal2));

        set_api_key_principals(vec![]);
        assert!(!is_api_key_principal(&principal1));
        assert!(!is_api_key_principal(&principal2));
    }
}
