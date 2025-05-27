use crate::{
    fixtures::nonce_account,
    nonce::{nonce_from_account, ExtractNonceError},
};
use assert_matches::assert_matches;
use serde_json::json;
use solana_account_decoder_client_types::{UiAccount, UiAccountData, UiAccountEncoding};
use solana_hash::Hash;
use std::str::FromStr;

mod durable_nonce {
    use super::*;

    #[test]
    fn should_extract_base64_encoded_durable_nonce() {
        let account = UiAccount::from(nonce_account());

        let durable_nonce = nonce_from_account(&account);

        assert_eq!(
            durable_nonce,
            Ok(Hash::from_str("6QK3LC8dsRtH2qVU47cSvgchPHNU72f1scvg2LuN2z7e").unwrap())
        )
    }

    #[test]
    fn should_extract_base58_encoded_durable_nonce() {
        let mut account = UiAccount::from(nonce_account());
        account.data = UiAccountData::Binary("df8aQUMTjFsfZ6gjD4sxzFKMXqaZEvX2G2ZZA79reSjPFCPVrPb5KBwJbXApxNhhC7HETRFukWRK8EYg2hQVj9L4AmTS5RvxYqFS8nDpvfhZ".to_string(), UiAccountEncoding::Base58);

        let durable_nonce = nonce_from_account(&account);

        assert_eq!(
            durable_nonce,
            Ok(Hash::from_str("6QK3LC8dsRtH2qVU47cSvgchPHNU72f1scvg2LuN2z7e").unwrap())
        )
    }

    #[test]
    fn should_fail_for_unsupported_encoding_format() {
        let account: UiAccount = serde_json::from_value(json!({
            "data": {
                "parsed": {
                    "info": {
                        "authority": "5CZKcm6PakaRWGK8NogzXvj8CjA71uSofKLohoNi4Wom",
                        "blockhash": "6QK3LC8dsRtH2qVU47cSvgchPHNU72f1scvg2LuN2z7e",
                        "feeCalculator": {
                            "lamportsPerSignature": "5000"
                        }
                    },
                    "type": "initialized"
                },
                "program": "nonce",
                "space": 80
            },
            "executable": false,
            "lamports": 1499900,
            "owner": "11111111111111111111111111111111",
            "rentEpoch": 18_446_744_073_709_551_615u128,
            "space": 80
        }))
        .unwrap();

        let durable_nonce = nonce_from_account(&account);

        assert_eq!(
            durable_nonce,
            Err(ExtractNonceError::UnsupportedEncodingFormat)
        )
    }

    #[test]
    fn should_fail_for_invalid_account_data() {
        let mut account = UiAccount::from(nonce_account());
        account.data = UiAccountData::Binary(
            "ARAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string(),
            UiAccountEncoding::Base64,
        );

        let durable_nonce = nonce_from_account(&account);

        assert_matches!(durable_nonce, Err(ExtractNonceError::InvalidAccountData(_)))
    }

    #[test]
    fn should_fail_for_uninitialized_account() {
        let mut account = UiAccount::from(nonce_account());
        account.data = UiAccountData::Binary(
            "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string(),
            UiAccountEncoding::Base64,
        );

        let durable_nonce = nonce_from_account(&account);

        assert_eq!(durable_nonce, Err(ExtractNonceError::Uninitialized))
    }

    #[test]
    fn should_fail_for_empty_account() {
        let mut account = UiAccount::from(nonce_account());
        account.data = UiAccountData::Binary("".to_string(), UiAccountEncoding::Base64);

        let durable_nonce = nonce_from_account(&account);

        assert_eq!(durable_nonce, Err(ExtractNonceError::UnexpectedDataSize(0)))
    }

    #[test]
    fn should_fail_for_invalid_account_owner() {
        let mut account = UiAccount::from(nonce_account());
        account.owner = "AAAGuCgkmxYDTiBvzx1QT5XEjqXPRtQaiEXQo4gatD2o".to_string();

        let durable_nonce = nonce_from_account(&account);

        assert_eq!(
            durable_nonce,
            Err(ExtractNonceError::InvalidAccountOwner(
                "AAAGuCgkmxYDTiBvzx1QT5XEjqXPRtQaiEXQo4gatD2o".to_string()
            ))
        )
    }
}
