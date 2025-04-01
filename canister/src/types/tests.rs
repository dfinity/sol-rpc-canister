use crate::{
    memory::{init_state, reset_state, State},
    providers::{resolve_rpc_provider, PROVIDERS},
    types::{ApiKey, OverrideProvider, RoundingError},
};
use proptest::{
    prelude::{prop, Strategy},
    proptest,
};
use sol_rpc_types::{HttpHeader, RegexSubstitution, RpcEndpoint, RpcSource};

mod override_provider_tests {
    use super::*;
    use sol_rpc_types::SupportedRpcProviderId;

    proptest! {
        #[test]
        fn should_override_provider_with_localhost(provider in arb_provider()) {
            with_api_key_for_provider(provider);
            let api = resolve_rpc_provider(RpcSource::Supported(provider));
            let overriden_provider  = override_to_localhost().apply(api);
            assert_eq!(
                overriden_provider,
                Ok(RpcEndpoint {
                    url: "http://localhost:8545".to_string(),
                    headers: None
                })
            );
        }
    }

    proptest! {
        #[test]
        fn should_be_noop_when_empty(provider in arb_provider()) {
            with_api_key_for_provider(provider);
            let no_override = OverrideProvider::default();
            let initial_api = resolve_rpc_provider(RpcSource::Supported(provider));
            let overriden_api = no_override.apply(initial_api.clone());
            assert_eq!(Ok(initial_api), overriden_api);
        }
    }

    proptest! {
        #[test]
        fn should_use_replacement_pattern(provider in arb_provider()) {
            with_api_key_for_provider(provider);
            let identity_override = OverrideProvider {
                override_url: Some(RegexSubstitution {
                    pattern: "(\\.com)".into(),
                    replacement: ".ch".to_string(),
                }),
            };
            let initial_api = resolve_rpc_provider(RpcSource::Supported(provider));
            let overriden_provider = identity_override.apply(initial_api.clone());
            assert_eq!(overriden_provider,
                Ok(RpcEndpoint {
                    url: initial_api.url.replace(".com", ".ch"),
                    headers: None,
                })
            );
        }
    }

    proptest! {
        #[test]
        fn should_override_headers(provider in arb_provider()) {
            with_api_key_for_provider(provider);
            let identity_override = OverrideProvider {
                override_url: Some(RegexSubstitution {
                    pattern: "(.*)".into(),
                    replacement: "$1".to_string(),
                }),
            };
            let api_with_headers = RpcEndpoint {
                headers: Some(vec![HttpHeader {
                    name: "key".to_string(),
                    value: "123".to_string(),
                }]),
                ..resolve_rpc_provider(RpcSource::Supported(provider))
            };
            let overriden_provider = identity_override.apply(api_with_headers.clone());
            assert_eq!(
                overriden_provider,
                Ok(RpcEndpoint {
                    url: api_with_headers.url,
                    headers: None
                })
            )
        }
    }

    fn with_api_key_for_provider(provider: SupportedRpcProviderId) {
        reset_state();
        let mut state = State::default();
        state.insert_api_key(
            provider,
            ApiKey::try_from("dummy_api_key".to_string()).unwrap(),
        );
        init_state(state);
    }

    fn override_to_localhost() -> OverrideProvider {
        OverrideProvider {
            override_url: Some(RegexSubstitution {
                pattern: "^https://.*".into(),
                replacement: "http://localhost:8545".to_string(),
            }),
        }
    }

    fn arb_provider() -> impl Strategy<Value = SupportedRpcProviderId> {
        prop::sample::select(
            PROVIDERS.with(|providers| providers.clone().into_keys().collect::<Vec<_>>()),
        )
    }
}
mod rounding_error_tests {
    use super::*;

    #[test]
    fn should_round_slot() {
        for (rounding_error, slot, rounded) in [
            (0, 0, 0),
            (0, 13, 13),
            (1, 13, 13),
            (10, 13, 10),
            (10, 100, 100),
            (10, 101, 100),
            (10, 102, 100),
            (10, 103, 100),
            (10, 104, 100),
            (10, 105, 100),
            (10, 106, 100),
            (10, 107, 100),
            (10, 108, 100),
            (10, 109, 100),
            (10, 110, 110),
        ] {
            assert_eq!(RoundingError::new(rounding_error).round(slot), rounded);
        }
    }

    proptest! {
        #[test]
        fn should_not_panic (rounding_error: u64, slot: u64) {
            let _result = RoundingError::new(rounding_error).round(slot);
        }
    }
}
