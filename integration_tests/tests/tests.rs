use sol_rpc_int_tests::{Setup, SolRpcTestClient, ADDITIONAL_TEST_ID};
use sol_rpc_types::{
    InstallArgs, Provider, RpcAccess, RpcAuth, RpcService, SolMainnetService, SolanaCluster,
};

mod get_provider_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_providers() {
        let setup = Setup::new().await;
        let client = setup.client();
        let providers = client.get_providers().await;

        assert_eq!(providers.len(), 5);

        assert_eq!(
            providers[0],
            Provider {
                provider_id: "alchemy-mainnet".to_string(),
                cluster: SolanaCluster::Mainnet,
                access: RpcAccess::Authenticated {
                    auth: RpcAuth::BearerToken {
                        url: "https://solana-mainnet.g.alchemy.com/v2".to_string(),
                    },
                    public_url: Some("https://solana-mainnet.g.alchemy.com/v2/demo".to_string()),
                },
                alias: Some(RpcService::SolMainnet(SolMainnetService::Alchemy)),
            }
        );

        setup.drop().await;
    }
}

mod retrieve_logs_tests {
    use super::*;

    #[tokio::test]
    async fn should_retrieve_logs() {
        let setup = Setup::new().await;
        let client = setup.client();
        assert_eq!(client.retrieve_logs("DEBUG").await, vec![]);
        assert_eq!(client.retrieve_logs("INFO").await, vec![]);

        // Generate some canlog
        setup
            .client()
            .with_caller(setup.controller())
            .update_api_keys(&[(
                "alchemy-mainnet".to_string(),
                Some("unauthorized-api-key".to_string()),
            )])
            .await;

        assert_eq!(client.retrieve_logs("DEBUG").await, vec![]);
        assert!(client.retrieve_logs("INFO").await[0]
            .message
            .contains("Updating API keys"));
    }
}

mod update_api_key_tests {
    use super::*;

    #[tokio::test]
    async fn should_update_api_key() {
        let authorized_caller = ADDITIONAL_TEST_ID;
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![authorized_caller]),
            ..Default::default()
        })
        .await;

        let provider_id = "alchemy-mainnet";
        let api_key = "test-api-key";
        let client = setup.client().with_caller(authorized_caller);
        client
            .update_api_keys(&[(provider_id.to_string(), Some(api_key.to_string()))])
            .await;
        client
            .verify_api_key((provider_id.to_string(), Some(api_key.to_string())))
            .await;

        client
            .update_api_keys(&[(provider_id.to_string(), None)])
            .await;
        client.verify_api_key((provider_id.to_string(), None)).await;
    }

    #[tokio::test]
    #[should_panic(expected = "You are not authorized")]
    async fn should_prevent_unauthorized_update_api_keys() {
        let setup = Setup::new().await;
        setup
            .client()
            .update_api_keys(&[(
                "alchemy-mainnet".to_string(),
                Some("unauthorized-api-key".to_string()),
            )])
            .await;
    }

    #[tokio::test]
    #[should_panic(expected = "Trying to set API key for unauthenticated provider")]
    async fn should_prevent_unauthenticated_update_api_keys() {
        let setup = Setup::new().await;
        setup
            .client()
            .with_caller(setup.controller())
            .update_api_keys(&[(
                "publicnode-mainnet".to_string(),
                Some("invalid-api-key".to_string()),
            )])
            .await;
    }
}

mod canister_upgrade_tests {
    use super::*;

    #[tokio::test]
    async fn upgrade_should_keep_api_keys() {
        let setup = Setup::new().await;
        let provider_id = "alchemy-mainnet";
        let api_key = "test-api-key";
        let client = setup.client().with_caller(setup.controller());
        client
            .update_api_keys(&[(provider_id.to_string(), Some(api_key.to_string()))])
            .await;
        client
            .verify_api_key((provider_id.to_string(), Some(api_key.to_string())))
            .await;

        setup.upgrade_canister(InstallArgs::default()).await;

        client
            .verify_api_key((provider_id.to_string(), Some(api_key.to_string())))
            .await;
    }

    #[tokio::test]
    async fn upgrade_should_keep_manage_api_key_principals() {
        let authorized_caller = ADDITIONAL_TEST_ID;
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![authorized_caller]),
            ..Default::default()
        })
        .await;
        setup
            .upgrade_canister(InstallArgs {
                manage_api_keys: None,
                ..Default::default()
            })
            .await;
        setup
            .client()
            .with_caller(authorized_caller)
            .update_api_keys(&[(
                "alchemy-mainnet".to_string(),
                Some("authorized-api-key".to_string()),
            )])
            .await;
    }

    #[tokio::test]
    #[should_panic(expected = "You are not authorized")]
    async fn upgrade_should_change_manage_api_key_principals() {
        let deauthorized_caller = ADDITIONAL_TEST_ID;
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![deauthorized_caller]),
            ..Default::default()
        })
        .await;
        setup
            .upgrade_canister(InstallArgs {
                manage_api_keys: Some(vec![]),
                ..Default::default()
            })
            .await;
        setup
            .client()
            .with_caller(deauthorized_caller)
            .update_api_keys(&[(
                "alchemy-mainnet".to_string(),
                Some("unauthorized-api-key".to_string()),
            )])
            .await;
    }
}

use canlog::LogFilter;
use std::cell::RefCell;

thread_local! {
    static LOG_FILTER: RefCell<LogFilter> = RefCell::default();
}

mod logging_tests {
    use super::*;
    use canlog::{log, GetLogFilter, Log, LogEntry, LogPriorityLevels, Sort};
    use canlog_derive::LogPriorityLevels;
    use proptest::{prop_assert, proptest};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Copy, Serialize, Deserialize, LogPriorityLevels)]
    enum TestPriority {
        #[log_level(capacity = 1000, name = "INFO_TEST")]
        Info,
    }

    impl GetLogFilter for TestPriority {
        fn get_log_filter() -> LogFilter {
            LOG_FILTER.with(|cell| cell.borrow().clone())
        }
    }

    fn set_log_filter(filter: LogFilter) {
        LOG_FILTER.set(filter);
    }

    fn info_log_entry_with_timestamp(timestamp: u64) -> LogEntry<TestPriority> {
        LogEntry {
            timestamp,
            priority: TestPriority::Info,
            file: String::default(),
            line: 0,
            message: String::default(),
            counter: 0,
        }
    }

    fn is_ascending(log: &Log<TestPriority>) -> bool {
        for i in 0..log.entries.len() - 1 {
            if log.entries[i].timestamp > log.entries[i + 1].timestamp {
                return false;
            }
        }
        true
    }

    fn is_descending(log: &Log<TestPriority>) -> bool {
        for i in 0..log.entries.len() - 1 {
            if log.entries[i].timestamp < log.entries[i + 1].timestamp {
                return false;
            }
        }
        true
    }

    fn get_messages() -> Vec<String> {
        canlog::export_logs(TestPriority::Info.get_buffer())
            .into_iter()
            .map(|entry| entry.message)
            .collect()
    }

    proptest! {
        #[test]
        fn logs_always_fit_in_message(
            number_of_entries in 1..100_usize,
            entry_size in 1..10000_usize,
            max_body_size in 100..10000_usize
        ) {
            let mut entries: Vec<LogEntry<TestPriority>> = vec![];
            for _ in 0..number_of_entries {
                entries.push(LogEntry {
                    timestamp: 0,
                    priority: TestPriority::Info,
                    file: String::default(),
                    line: 0,
                    message: "1".repeat(entry_size),
                    counter: 0,
                });
            }
            let log = Log { entries };
            let truncated_logs_json_len = log.serialize_logs(max_body_size).len();
            prop_assert!(truncated_logs_json_len <= max_body_size);
        }
    }

    #[test]
    fn sorting_order() {
        let mut log = Log { entries: vec![] };
        log.entries.push(info_log_entry_with_timestamp(2));
        log.entries.push(info_log_entry_with_timestamp(0));
        log.entries.push(info_log_entry_with_timestamp(1));

        log.sort_logs(Sort::Ascending);
        assert!(is_ascending(&log));

        log.sort_logs(Sort::Descending);
        assert!(is_descending(&log));
    }

    #[test]
    fn simple_logs_truncation() {
        let mut entries: Vec<LogEntry<TestPriority>> = vec![];
        const MAX_BODY_SIZE: usize = 3_000_000;

        for _ in 0..10 {
            entries.push(LogEntry {
                timestamp: 0,
                priority: TestPriority::Info,
                file: String::default(),
                line: 0,
                message: String::default(),
                counter: 0,
            });
        }
        let log = Log {
            entries: entries.clone(),
        };
        let small_len = serde_json::to_string(&log).unwrap_or_default().len();

        entries.push(LogEntry {
            timestamp: 0,
            priority: TestPriority::Info,
            file: String::default(),
            line: 0,
            message: "1".repeat(MAX_BODY_SIZE),
            counter: 0,
        });
        let log = Log { entries };
        let entries_json = serde_json::to_string(&log).unwrap_or_default();
        assert!(entries_json.len() > MAX_BODY_SIZE);

        let truncated_logs_json = log.serialize_logs(MAX_BODY_SIZE);

        assert_eq!(small_len, truncated_logs_json.len());
    }

    #[test]
    fn one_entry_too_big() {
        let mut entries: Vec<LogEntry<TestPriority>> = vec![];
        const MAX_BODY_SIZE: usize = 3_000_000;

        entries.push(LogEntry {
            timestamp: 0,
            priority: TestPriority::Info,
            file: String::default(),
            line: 0,
            message: "1".repeat(MAX_BODY_SIZE),
            counter: 0,
        });
        let log = Log { entries };
        let truncated_logs_json_len = log.serialize_logs(MAX_BODY_SIZE).len();
        assert!(truncated_logs_json_len < MAX_BODY_SIZE);
        assert_eq!("{\"entries\":[]}", log.serialize_logs(MAX_BODY_SIZE));
    }

    #[test]
    fn should_truncate_last_entry() {
        let log_entries = vec![
            info_log_entry_with_timestamp(0),
            info_log_entry_with_timestamp(1),
            info_log_entry_with_timestamp(2),
        ];
        let log_with_2_entries = Log {
            entries: {
                let mut entries = log_entries.clone();
                entries.pop();
                entries
            },
        };
        let log_with_3_entries = Log {
            entries: log_entries,
        };

        let serialized_log_with_2_entries = log_with_2_entries.serialize_logs(usize::MAX);
        let serialized_log_with_3_entries =
            log_with_3_entries.serialize_logs(serialized_log_with_2_entries.len());

        assert_eq!(serialized_log_with_3_entries, serialized_log_with_2_entries);
    }

    #[test]
    fn should_show_all() {
        set_log_filter(LogFilter::ShowAll);
        log!(TestPriority::Info, "ABC");
        log!(TestPriority::Info, "123");
        log!(TestPriority::Info, "!@#");
        assert_eq!(get_messages(), vec!["ABC", "123", "!@#"]);
    }

    #[test]
    fn should_hide_all() {
        set_log_filter(LogFilter::HideAll);
        log!(TestPriority::Info, "ABC");
        log!(TestPriority::Info, "123");
        log!(TestPriority::Info, "!@#");
        assert_eq!(get_messages().len(), 0);
    }

    #[test]
    fn should_show_pattern() {
        set_log_filter(LogFilter::ShowPattern("end$".into()));
        log!(TestPriority::Info, "message");
        log!(TestPriority::Info, "message end");
        log!(TestPriority::Info, "end message");
        assert_eq!(get_messages(), vec!["message end"]);
    }

    #[test]
    fn should_hide_pattern_including_message_type() {
        set_log_filter(LogFilter::ShowPattern("^INFO_TEST [^ ]* 123".into()));
        log!(TestPriority::Info, "123");
        log!(TestPriority::Info, "INFO_TEST 123");
        log!(TestPriority::Info, "");
        log!(TestPriority::Info, "123456");
        assert_eq!(get_messages(), vec!["123", "123456"]);
    }

    #[test]
    fn should_hide_pattern() {
        set_log_filter(LogFilter::HidePattern("[ABC]".into()));
        log!(TestPriority::Info, "remove A");
        log!(TestPriority::Info, "...B...");
        log!(TestPriority::Info, "C");
        log!(TestPriority::Info, "message");
        assert_eq!(get_messages(), vec!["message"]);
    }
}
