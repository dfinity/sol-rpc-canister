use crate::VecWithMaxLen;
use candid::{CandidType, Decode, Encode};
use proptest::arbitrary::Arbitrary;
use proptest::prelude::any;
use proptest::prelude::TestCaseError;
use proptest::prop_assert;
use proptest::{
    prelude::{prop, Strategy},
    prop_assert_eq, proptest,
};
use serde::de::DeserializeOwned;

mod vec_with_max_len_tests {
    use super::*;

    proptest! {
        #[test]
        fn should_encode_decode (
            string_vec in arb_vec_with_max_size(50),
            int_vec in arb_vec_with_max_size(75),
            bytes_vec in arb_vec_with_max_size(100))
        {
            encode_decode_roundtrip::<String, 50>(string_vec)?;
            encode_decode_roundtrip::<i32, 75>(int_vec)?;
            encode_decode_roundtrip::<Vec<u8>, 100>(bytes_vec)?;
        }

        #[test]
        fn should_fail_to_decode_when_too_long(
            string_vec in arb_vec_with_min_size(50),
            int_vec in arb_vec_with_min_size(75),
            bytes_vec in arb_vec_with_min_size(100))
        {
            expect_decoding_error::<String, 50>(string_vec)?;
            expect_decoding_error::<i32, 75>(int_vec)?;
            expect_decoding_error::<Vec<u8>, 100>(bytes_vec)?;
        }
    }

    fn encode_decode_roundtrip<T, const CAPACITY: usize>(value: Vec<T>) -> Result<(), TestCaseError>
    where
        T: Clone + CandidType + std::fmt::Debug + PartialEq + DeserializeOwned,
    {
        let parsed_value: VecWithMaxLen<T, CAPACITY> = TryFrom::try_from(value.clone())?;
        let encoded_value = Encode!(&value)?;
        let encoded_parsed_value = Encode!(&parsed_value)?;
        prop_assert_eq!(
            &encoded_value,
            &encoded_parsed_value,
            "Encoded value differ for {:?}",
            value
        );

        let decoded_text_value = Decode!(&encoded_value, VecWithMaxLen<T, CAPACITY>)?;
        prop_assert_eq!(
            &decoded_text_value,
            &parsed_value,
            "Decoded value differ for {:?}",
            value
        );
        Ok(())
    }

    fn expect_decoding_error<T, const CAPACITY: usize>(
        too_long: Vec<T>,
    ) -> Result<(), TestCaseError>
    where
        T: Clone + CandidType + std::fmt::Debug + PartialEq + DeserializeOwned,
    {
        let result = Decode!(&Encode!(&too_long)?, VecWithMaxLen<T, CAPACITY>);
        prop_assert!(
            result.is_err(),
            "Expected error decoding {:?}, got: {:?}",
            too_long,
            result
        );
        Ok(())
    }

    fn arb_vec_with_max_size<T: Arbitrary>(max_size: usize) -> impl Strategy<Value = Vec<T>> {
        prop::collection::vec(any::<T>(), 0..=max_size)
    }

    fn arb_vec_with_min_size<T: Arbitrary>(min_size: usize) -> impl Strategy<Value = Vec<T>> {
        prop::collection::vec(any::<T>(), min_size + 1..=min_size + 100)
    }
}
