use crate::{
    fixtures::{initialized_nonce_account, usdc_account},
    nonce::{extract_durable_nonce, ExtractNonceError},
};
use assert_matches::assert_matches;
use serde_json::json;
use solana_account_decoder_client_types::{UiAccount, UiAccountData, UiAccountEncoding};
use solana_hash::Hash;
use solana_rpc_client_nonce_utils::Error;
use std::str::FromStr;

mod durable_nonce {
    use super::*;

    #[test]
    fn should_extract_base64_encoded_durable_nonce() {
        let account = UiAccount::from(initialized_nonce_account());

        let durable_nonce = extract_durable_nonce(&account);

        assert_eq!(
            durable_nonce,
            Ok(Hash::from_str("6QK3LC8dsRtH2qVU47cSvgchPHNU72f1scvg2LuN2z7e").unwrap())
        )
    }

    #[test]
    fn should_extract_base58_encoded_durable_nonce() {
        let account = UiAccount {
            lamports: 1_499_900,
            data: UiAccountData::Binary("df8aQUMTjFsfZ6gjD4sxzFKMXqaZEvX2G2ZZA79reSjPFCPVrPb5KBwJbXApxNhhC7HETRFukWRK8EYg2hQVj9L4AmTS5RvxYqFS8nDpvfhZ".to_string(), UiAccountEncoding::Base58),
            owner: "11111111111111111111111111111111".to_string(),
            executable: false,
            rent_epoch: 18_446_744_073_709_551_615,
            space: Some(80)
        };

        let durable_nonce = extract_durable_nonce(&account);

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

        let durable_nonce = extract_durable_nonce(&account);

        assert_eq!(durable_nonce, Err(ExtractNonceError::AccountDecodingError))
    }

    #[test]
    fn should_fail_for_invalid_nonce_account() {
        let account = UiAccount::from(usdc_account());

        let durable_nonce = extract_durable_nonce(&account);

        assert_matches!(
            durable_nonce,
            Err(ExtractNonceError::DurableNonceError(
                Error::InvalidAccountOwner
            ))
        );
    }

    #[test]
    fn should_fail_for_uninitialized_account() {
        let account = UiAccount {
            lamports: 1_500_000,
            data: UiAccountData::Binary(
                "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string(),
                UiAccountEncoding::Base64,
            ),
            owner: "11111111111111111111111111111111".to_string(),
            executable: false,
            rent_epoch: 18_446_744_073_709_551_615,
            space: Some(80),
        };

        let durable_nonce = extract_durable_nonce(&account);

        assert_matches!(
            durable_nonce,
            Err(ExtractNonceError::DurableNonceError(
                Error::InvalidStateForOperation
            ))
        );
    }
}
