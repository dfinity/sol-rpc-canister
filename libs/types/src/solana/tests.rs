mod impl_candid {
    use crate::{Hash, Pubkey, Signature};
    use candid::{CandidType, Decode, Encode};
    use proptest::{
        collection::SizeRange,
        prelude::{any, Strategy, TestCaseError},
        prop_assert, prop_assert_eq, proptest,
    };
    use serde::de::DeserializeOwned;
    use std::str::FromStr;

    proptest! {
        #[test]
        fn should_encode_decode(
            bs58_32 in arb_var_len_base58_string(32),
            bs58_64 in arb_var_len_base58_string(64)
        ) {
            encode_decode_roundtrip::<Pubkey>(&bs58_32)?;
            encode_decode_roundtrip::<Signature>(&bs58_64)?;
            encode_decode_roundtrip::<Hash>(&bs58_32)?;
        }

        #[test]
        fn should_fail_to_decode_strings_with_wrong_length(
            short_bs58_32 in arb_var_len_base58_string(0..=31),
            long_bs58_32 in arb_var_len_base58_string(33..=100),
            short_bs58_64 in arb_var_len_base58_string(0..=63),
            long_bs58_64 in arb_var_len_base58_string(65..=200)
        ){
            expect_decoding_error::<Pubkey>(&short_bs58_32)?;
            expect_decoding_error::<Pubkey>(&long_bs58_32)?;
            expect_decoding_error::<Hash>(&short_bs58_32)?;
            expect_decoding_error::<Hash>(&long_bs58_32)?;
            expect_decoding_error::<Signature>(&short_bs58_64)?;
            expect_decoding_error::<Signature>(&long_bs58_64)?;
        }

        #[test]
        fn should_fail_when_invalid_characters(
            mut bs58_32 in arb_var_len_base58_string(32),
            mut bs58_64 in arb_var_len_base58_string(64),
            index in any::<usize>(),
            non_bs58_char in "[0IOl]{1}"
        ) {
            let replacement_index= index % bs58_32.len();
            bs58_32.replace_range(replacement_index..=replacement_index, &non_bs58_char);
            expect_decoding_error::<Pubkey>(&bs58_32)?;
            expect_decoding_error::<Hash>(&bs58_32)?;

            let replacement_index= index % bs58_64.len();
            bs58_64.replace_range(replacement_index..=replacement_index, &non_bs58_char);
            expect_decoding_error::<Signature>(&bs58_64)?;
        }

    }

    fn encode_decode_roundtrip<T>(value: &str) -> Result<(), TestCaseError>
    where
        T: FromStr + CandidType + DeserializeOwned + PartialEq + std::fmt::Debug,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let parsed_value: T = value.parse().unwrap();
        let encoded_text_value = Encode!(&value).unwrap();
        let encoded_parsed_value = Encode!(&parsed_value).unwrap();
        prop_assert_eq!(
            &encoded_text_value,
            &encoded_parsed_value,
            "Encoded value differ for {}",
            value
        );

        let decoded_text_value = Decode!(&encoded_text_value, T).unwrap();
        prop_assert_eq!(
            &decoded_text_value,
            &parsed_value,
            "Decoded value differ for {}",
            value
        );
        Ok(())
    }

    fn expect_decoding_error<T>(wrong_base58: &str) -> Result<(), TestCaseError>
    where
        T: FromStr + CandidType + DeserializeOwned + PartialEq + std::fmt::Debug,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let result = Decode!(&Encode!(&wrong_base58).unwrap(), T);
        prop_assert!(
            result.is_err(),
            "Expected error decoding {}, got: {:?}",
            wrong_base58,
            result
        );
        Ok(())
    }

    fn arb_var_len_base58_string(
        num_bytes_range: impl Into<SizeRange>,
    ) -> impl Strategy<Value = String> {
        proptest::collection::vec(any::<u8>(), num_bytes_range)
            .prop_map(|bytes| bs58::encode(bytes).into_string())
    }
}
