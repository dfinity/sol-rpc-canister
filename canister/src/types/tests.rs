use crate::{
    providers::PROVIDERS,
    rpc_client::get_api,
    state::{init_state, State},
    types::{ApiKey, OverrideProvider},
};
use proptest::{
    prelude::{prop, Strategy},
    proptest,
};
use sol_rpc_types::{HttpHeader, Provider, RegexSubstitution, RpcApi};

mod override_provider_tests {
    use super::*;
    use crate::state::reset_state;

    proptest! {
        #[test]
        fn should_override_provider_with_localhost(provider in arb_provider()) {
            with_api_key_for_provider(&provider);
            let api = get_api(&provider);
            let overriden_provider  = override_to_localhost().apply(api);
            assert_eq!(
                overriden_provider,
                Ok(RpcApi {
                    url: "http://localhost:8545".to_string(),
                    headers: None
                })
            );
        }
    }

    proptest! {
        #[test]
        fn should_be_noop_when_empty(provider in arb_provider()) {
            with_api_key_for_provider(&provider);
            let no_override = OverrideProvider::default();
            let initial_api = get_api(&provider);
            let overriden_api = no_override.apply(initial_api.clone());
            assert_eq!(Ok(initial_api), overriden_api);
        }
    }

    proptest! {
        #[test]
        fn should_use_replacement_pattern(provider in arb_provider()) {
            with_api_key_for_provider(&provider);
            let identity_override = OverrideProvider {
                override_url: Some(RegexSubstitution {
                    pattern: "(\\.com)".into(),
                    replacement: ".ch".to_string(),
                }),
            };
            let initial_api = get_api(&provider);
            let overriden_provider = identity_override.apply(initial_api.clone());
            assert_eq!(overriden_provider,
                Ok(RpcApi {
                    url: initial_api.url.replace(".com", ".ch"),
                    headers: None,
                })
            );
        }
    }

    proptest! {
        #[test]
        fn should_override_headers(provider in arb_provider()) {
            with_api_key_for_provider(&provider);
            let identity_override = OverrideProvider {
                override_url: Some(RegexSubstitution {
                    pattern: "(.*)".into(),
                    replacement: "$1".to_string(),
                }),
            };
            let api_with_headers = RpcApi {
                headers: Some(vec![HttpHeader {
                    name: "key".to_string(),
                    value: "123".to_string(),
                }]),
                ..get_api(&provider)
            };
            let overriden_provider = identity_override.apply(api_with_headers.clone());
            assert_eq!(
                overriden_provider,
                Ok(RpcApi {
                    url: api_with_headers.url,
                    headers: None
                })
            )
        }
    }

    fn with_api_key_for_provider(provider: &Provider) {
        reset_state();
        let mut state = State::default();
        state.insert_api_key(
            provider.provider_id.clone(),
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

    fn arb_provider() -> impl Strategy<Value = Provider> {
        prop::sample::select(PROVIDERS.with(|providers| providers.to_vec()))
    }
}
