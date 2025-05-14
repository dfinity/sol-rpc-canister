use crate::VecWithMaxLen;
use candid::{Decode, Encode};
use proptest::{
    arbitrary::any,
    prelude::{prop, Strategy},
    proptest,
};
use serde::Deserialize;
use serde_json::json;

proptest! {
    #[test]
    fn should_encode_decode (values in arb_vec_with_capacity()) {
        let encoded = Encode!(&values).unwrap();
        let decoded = Decode!(&encoded, VecWithMaxLen::<String, 100>).unwrap();

        assert_eq!(decoded, values);
    }

    #[test]
    fn should_deserialize(values in prop::collection::vec(any::<String>(), 0..100)) {
        let serialized = json!(values);

        let result = VecWithMaxLen::<String, 100>::deserialize(&serialized);

        assert!(result.is_ok());
        assert_eq!(Vec::from(result.unwrap()), values);
    }

    #[test]
    fn should_not_deserialize(values in prop::collection::vec(any::<String>(), 101..1000)) {
        let serialized = json!(values);

        let result = VecWithMaxLen::<String, 100>::deserialize(&serialized);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!("Validation error: Expected at most 100 items, but got {}", values.len())
        );
    }
}

fn arb_vec_with_capacity<const CAPACITY: usize>(
) -> impl Strategy<Value = VecWithMaxLen<String, CAPACITY>> {
    prop::collection::vec(any::<String>(), 0..=CAPACITY)
        .prop_map(|values| values.try_into().unwrap())
}
