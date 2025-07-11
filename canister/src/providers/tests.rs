use super::{Providers, PROVIDERS};
use crate::constants::API_KEY_REPLACE_STRING;
use sol_rpc_types::{RpcAccess, RpcAuth, SupportedRpcProvider, SupportedRpcProviderId};
use std::collections::BTreeSet;
use strum::IntoEnumIterator;

#[test]
fn test_rpc_provider_url_patterns() {
    PROVIDERS.with(|providers| {
        for (provider, SupportedRpcProvider { access, .. }) in providers {
            fn assert_not_url_pattern(url: &str, provider: &SupportedRpcProviderId) {
                assert!(
                    !url.contains(API_KEY_REPLACE_STRING),
                    "Unexpected API key in URL for provider: {:?}",
                    provider
                )
            }
            fn assert_url_pattern(url: &str, provider: &SupportedRpcProviderId) {
                assert!(
                    url.contains(API_KEY_REPLACE_STRING),
                    "Missing API key in URL pattern for provider: {:?}",
                    provider
                )
            }
            match access {
                RpcAccess::Authenticated { auth, public_url } => {
                    match auth {
                        RpcAuth::BearerToken { url } => assert_not_url_pattern(url, provider),
                        RpcAuth::UrlParameter { url_pattern } => {
                            assert_url_pattern(url_pattern, provider)
                        }
                    }
                    if let Some(public_url) = public_url {
                        assert_not_url_pattern(public_url, provider);
                    }
                }
                RpcAccess::Unauthenticated { public_url } => {
                    assert_not_url_pattern(public_url, provider);
                }
            }
        }
    })
}

#[test]
fn should_have_consistent_name_for_cluster() {
    PROVIDERS.with(|providers| {
        for (provider_id, provider) in providers {
            assert!(provider_id
                .to_string()
                .ends_with(&provider.cluster.to_string()));
        }
    })
}

#[test]
fn should_partition_providers_between_solana_cluster() {
    let mainnet_providers: BTreeSet<_> = Providers::MAINNET_PROVIDERS.iter().collect();
    let devnet_providers: BTreeSet<_> = Providers::DEVNET_PROVIDERS.iter().collect();
    let common_providers: BTreeSet<_> = mainnet_providers.intersection(&devnet_providers).collect();
    assert_eq!(common_providers, BTreeSet::default());

    let all_providers: BTreeSet<_> = SupportedRpcProviderId::iter().collect();
    let partitioned_providers: BTreeSet<_> = mainnet_providers
        .into_iter()
        .chain(devnet_providers)
        .copied()
        .collect();

    assert_eq!(all_providers, partitioned_providers);
}

mod providers_new {
    use crate::providers::Providers;
    use assert_matches::assert_matches;
    use canhttp::multi::Timestamp;
    use maplit::btreeset;
    use sol_rpc_types::{
        ConsensusStrategy, ProviderError, RpcSource, RpcSources, SolanaCluster,
        SupportedRpcProviderId,
    };

    #[test]
    fn should_fail_when_providers_explicitly_set_to_empty() {
        assert_matches!(
            Providers::new(
                RpcSources::Custom(vec![]),
                ConsensusStrategy::default(),
                Timestamp::UNIX_EPOCH
            ),
            Err(ProviderError::InvalidRpcConfig(_))
        );
    }

    #[test]
    fn should_use_default_providers() {
        for cluster in [SolanaCluster::Mainnet, SolanaCluster::Devnet] {
            let providers = Providers::new(
                RpcSources::Default(cluster),
                ConsensusStrategy::default(),
                Timestamp::UNIX_EPOCH,
            )
            .unwrap();
            assert!(!providers.sources.is_empty());
        }
    }

    #[test]
    fn should_use_specified_provider() {
        let provider1 = SupportedRpcProviderId::AlchemyMainnet;
        let provider2 = SupportedRpcProviderId::PublicNodeMainnet;

        let providers = Providers::new(
            RpcSources::Custom(vec![
                RpcSource::Supported(provider1),
                RpcSource::Supported(provider2),
            ]),
            ConsensusStrategy::default(),
            Timestamp::UNIX_EPOCH,
        )
        .unwrap();

        assert_eq!(
            providers.sources,
            btreeset! {
                RpcSource::Supported(provider1),
                RpcSource::Supported(provider2),
            }
        );
    }
}

mod supported_rpc_provider_usage {
    use crate::providers::{Providers, SupportedRpcProviderUsage};
    use canhttp::multi::Timestamp;
    use sol_rpc_types::{SolanaCluster, SupportedRpcProviderId};
    use std::time::Duration;

    const MINUTE: Duration = Duration::from_secs(60);

    #[test]
    fn should_have_default_ordering_when_no_data() {
        let mut usage = SupportedRpcProviderUsage::default();

        for (_cluster, providers) in all_supported_providers() {
            let ordered = usage.rank_ascending_evict(providers, Timestamp::UNIX_EPOCH);
            assert_eq!(ordered, providers);
        }
    }

    #[test]
    fn should_have_default_ordering_when_data_expired() {
        let mut usage = SupportedRpcProviderUsage::default();
        let now = Timestamp::UNIX_EPOCH;
        for (_cluster, supported_providers) in all_supported_providers() {
            let last_provider = *supported_providers.last().unwrap();
            usage.record_evict(last_provider, now);
        }

        let expired = Timestamp::from_unix_epoch(21 * MINUTE);
        for (_cluster, supported_providers) in all_supported_providers() {
            let ordered = usage.rank_ascending_evict(supported_providers, expired);
            assert_eq!(ordered, supported_providers);
        }
    }

    #[test]
    fn should_rank_based_on_non_expired_data() {
        let mut usage = SupportedRpcProviderUsage::default();
        for (_cluster, supported_providers) in all_supported_providers() {
            assert!(supported_providers.len() >= 2);

            // 3 entries, 2 expire after > 20 minutes
            usage.record_evict(supported_providers[0], Timestamp::UNIX_EPOCH);
            usage.record_evict(supported_providers[0], Timestamp::UNIX_EPOCH);
            usage.record_evict(supported_providers[0], Timestamp::from_unix_epoch(MINUTE));

            // 3 entries, 1 expire after > 20 minutes
            usage.record_evict(supported_providers[1], Timestamp::UNIX_EPOCH);
            usage.record_evict(supported_providers[1], Timestamp::from_unix_epoch(MINUTE));
            usage.record_evict(supported_providers[1], Timestamp::from_unix_epoch(MINUTE));
        }

        for (_cluster, supported_providers) in all_supported_providers() {
            let non_expired = Timestamp::from_unix_epoch(20 * MINUTE);
            let usage_before = usage.clone();
            let ordered = usage.rank_ascending_evict(supported_providers, non_expired);
            assert_eq!(ordered, supported_providers);
            assert_eq!(usage, usage_before);

            let expired = Timestamp::from_unix_epoch(21 * MINUTE);
            let usage_before = usage.clone();
            let ordered = usage.rank_ascending_evict(supported_providers, expired);
            let expected_order = {
                let mut expected = vec![supported_providers[1], supported_providers[0]];
                expected.extend(&supported_providers[2..]);
                expected
            };
            assert_eq!(ordered, expected_order);
            assert_ne!(usage, usage_before);
        }
    }

    fn all_supported_providers() -> [(SolanaCluster, &'static [SupportedRpcProviderId]); 2] {
        [
            (SolanaCluster::Mainnet, Providers::MAINNET_PROVIDERS),
            (SolanaCluster::Devnet, Providers::DEVNET_PROVIDERS),
        ]
    }
}
