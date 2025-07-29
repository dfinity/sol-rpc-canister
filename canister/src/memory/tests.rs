use crate::{
    memory::{decode, encode, init_state, mutate_state, next_request_id, read_state, State},
    types::{ApiKey, OverrideProvider},
};
use candid::Principal;
use canlog::LogFilter;
use proptest::{
    arbitrary::any,
    prelude::{prop, Just, Strategy},
    prop_oneof, proptest,
};
use serde::{Deserialize, Serialize};
use sol_rpc_types::{Mode, RegexString, RegexSubstitution, SupportedRpcProviderId};
use std::collections::{BTreeMap, BTreeSet};
use strum::IntoEnumIterator;

mod api_key_tests {
    use super::*;

    #[test]
    fn test_api_key_principals() {
        init_state(State::default());

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

    fn set_api_key_principals(new_principals: Vec<Principal>) {
        mutate_state(|state| state.set_api_key_principals(new_principals));
    }

    fn is_api_key_principal(principal: &Principal) -> bool {
        read_state(|state| state.is_api_key_principal(principal))
    }
}

mod request_counter_tests {
    use super::*;

    #[test]
    fn should_increment_request_id() {
        let request_ids = (0..10)
            .map(|_| next_request_id().to_string())
            .collect::<BTreeSet<_>>();
        assert_eq!(request_ids.len(), 10);
    }
}

mod upgrade_state_tests {
    use super::*;
    use crate::constants::VALID_API_KEY_CHARS;

    proptest! {
        #[test]
        fn should_decode_state(state in arb_state()) {
            let encoded = encode(&state);
            let decoded = decode::<State>(encoded.as_slice());
            assert_eq!(State::from(state), decoded);
        }
    }

    #[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    enum VersionedState {
        V0 {
            api_keys: BTreeMap<SupportedRpcProviderId, ApiKey>,
            api_key_principals: Vec<Principal>,
            override_provider: OverrideProvider,
            log_filter: LogFilter,
            mode: Mode,
            num_subnet_nodes: u32,
        },
        // Added optional `base_http_outcall_fee` field
        V1 {
            api_keys: BTreeMap<SupportedRpcProviderId, ApiKey>,
            api_key_principals: Vec<Principal>,
            override_provider: OverrideProvider,
            log_filter: LogFilter,
            mode: Mode,
            num_subnet_nodes: u32,
            base_http_outcall_fee: Option<u128>,
        },
    }

    impl From<VersionedState> for State {
        fn from(state: VersionedState) -> State {
            match state {
                VersionedState::V0 {
                    api_keys,
                    api_key_principals,
                    override_provider,
                    log_filter,
                    mode,
                    num_subnet_nodes,
                } => Self {
                    api_keys,
                    api_key_principals,
                    override_provider,
                    log_filter,
                    mode,
                    num_subnet_nodes,
                    base_http_outcall_fee: None,
                },
                VersionedState::V1 {
                    api_keys,
                    api_key_principals,
                    override_provider,
                    log_filter,
                    mode,
                    num_subnet_nodes,
                    base_http_outcall_fee,
                } => Self {
                    api_keys,
                    api_key_principals,
                    override_provider,
                    log_filter,
                    mode,
                    num_subnet_nodes,
                    base_http_outcall_fee,
                },
            }
        }
    }

    fn arb_state() -> impl Strategy<Value = VersionedState> {
        prop_oneof![arb_state_v0(), arb_state_v1()]
    }

    fn arb_state_v0() -> impl Strategy<Value = VersionedState> {
        (
            arb_api_keys(),
            arb_api_key_principals(),
            arb_override_provider(),
            arb_log_filter(),
            arb_mode(),
            any::<u32>(),
        )
            .prop_map(
                |(
                    api_keys,
                    api_key_principals,
                    override_provider,
                    log_filter,
                    mode,
                    num_subnet_nodes,
                )| VersionedState::V0 {
                    api_keys,
                    api_key_principals,
                    override_provider,
                    log_filter,
                    mode,
                    num_subnet_nodes,
                },
            )
    }

    fn arb_state_v1() -> impl Strategy<Value = VersionedState> {
        (
            arb_api_keys(),
            arb_api_key_principals(),
            arb_override_provider(),
            arb_log_filter(),
            arb_mode(),
            any::<u32>(),
            proptest::option::of(any::<u128>()),
        )
            .prop_map(
                |(
                    api_keys,
                    api_key_principals,
                    override_provider,
                    log_filter,
                    mode,
                    num_subnet_nodes,
                    base_http_outcall_fee,
                )| VersionedState::V1 {
                    api_keys,
                    api_key_principals,
                    override_provider,
                    log_filter,
                    mode,
                    num_subnet_nodes,
                    base_http_outcall_fee,
                },
            )
    }

    fn arb_mode() -> impl Strategy<Value = Mode> {
        prop::sample::select(Mode::iter().collect::<Vec<_>>())
    }

    fn arb_api_key_principals() -> impl Strategy<Value = Vec<Principal>> {
        prop::collection::vec(arb_principal(), 0..10)
    }

    fn arb_api_keys() -> impl Strategy<Value = BTreeMap<SupportedRpcProviderId, ApiKey>> {
        prop::collection::btree_map(
            prop::sample::select(SupportedRpcProviderId::iter().collect::<Vec<_>>()),
            arb_api_key(),
            0..=SupportedRpcProviderId::iter().count(),
        )
    }

    pub fn arb_api_key() -> impl Strategy<Value = ApiKey> {
        proptest::collection::vec(
            prop::sample::select(VALID_API_KEY_CHARS.chars().collect::<Vec<_>>()),
            1..=20,
        )
        .prop_map(String::from_iter)
        .prop_map(|value| ApiKey::try_from(value).unwrap())
    }

    pub fn arb_principal() -> impl Strategy<Value = Principal> {
        prop::collection::vec(any::<u8>(), 0..=29).prop_map(|bytes| Principal::from_slice(&bytes))
    }

    fn arb_regex_substitution() -> impl Strategy<Value = RegexSubstitution> {
        (".*".prop_map(RegexString), ".*").prop_map(|(pattern, replacement)| RegexSubstitution {
            pattern,
            replacement,
        })
    }

    fn arb_log_filter() -> impl Strategy<Value = LogFilter> {
        prop_oneof![
            Just(LogFilter::ShowAll),
            Just(LogFilter::HideAll),
            ".*".prop_map(canlog::RegexString)
                .prop_map(LogFilter::ShowPattern),
            ".*".prop_map(canlog::RegexString)
                .prop_map(LogFilter::HidePattern),
        ]
    }

    fn arb_override_provider() -> impl Strategy<Value = OverrideProvider> {
        proptest::option::of(arb_regex_substitution())
            .prop_map(|override_url| OverrideProvider { override_url })
    }
}
