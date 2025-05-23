use crate::{HttpHeader, RpcEndpoint};
use candid::{CandidType, Decode, Encode};
use proptest::{prelude::TestCaseError, prop_assert_eq};
use serde::de::DeserializeOwned;

#[test]
fn should_contain_host_without_sensitive_information() {
    for provider in [
        RpcEndpoint {
            url: "https://solana-mainnet.g.alchemy.com/v2".to_string(),
            headers: None,
        },
        RpcEndpoint {
            url: "https://solana-mainnet.g.alchemy.com/v2/key".to_string(),
            headers: None,
        },
        RpcEndpoint {
            url: "https://solana-mainnet.g.alchemy.com/v2".to_string(),
            headers: Some(vec![HttpHeader {
                name: "authorization".to_string(),
                value: "Bearer key".to_string(),
            }]),
        },
    ] {
        let debug = format!("{:?}", provider);
        assert_eq!(
            debug,
            "RpcApi { host: solana-mainnet.g.alchemy.com, url/headers: *** }"
        );
    }
}

mod rounding_error_tests {
    use crate::{rpc_client::tests::encode_decode_roundtrip, RoundingError};
    use proptest::proptest;

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

        #[test]
        fn should_encode_decode (rounding_error: u64) {
            encode_decode_roundtrip(RoundingError::new(rounding_error), rounding_error)?;
        }

    }
}

mod non_zero_u8 {
    use crate::rpc_client::{tests::encode_decode_roundtrip, NonZeroU8};
    use candid::{Decode, Encode};
    use proptest::proptest;

    proptest! {
        #[test]
        fn should_encode_decode(v in 1..255_u8) {
                encode_decode_roundtrip(NonZeroU8::try_from(v).unwrap(), v)?;
        }
    }

    #[test]
    fn should_fail_deserialization_when_zero() {
        let encoded_zero = Encode!(&0_u8).unwrap();
        assert!(Decode!(&encoded_zero, NonZeroU8).is_err());
    }
}

fn encode_decode_roundtrip<T, U>(wrapped_value: T, inner_value: U) -> Result<(), TestCaseError>
where
    T: CandidType + DeserializeOwned + PartialEq + std::fmt::Debug,
    U: CandidType,
{
    let encoded_wrapped_value = Encode!(&wrapped_value).unwrap();
    let encoded_inner_value = Encode!(&inner_value).unwrap();
    prop_assert_eq!(
        &encoded_wrapped_value,
        &encoded_inner_value,
        "Encoded value differ for {:?}",
        wrapped_value
    );

    let decoded_wrapped_value = Decode!(&encoded_wrapped_value, T).unwrap();
    prop_assert_eq!(
        &decoded_wrapped_value,
        &wrapped_value,
        "Decoded value differ for {:?}",
        wrapped_value
    );
    Ok(())
}
