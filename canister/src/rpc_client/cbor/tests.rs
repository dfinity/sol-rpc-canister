use minicbor::{Decode, Encode};
use proptest::prelude::{any, TestCaseError};
use proptest::{prop_assert_eq, proptest};
use sol_rpc_types::RoundingError;

proptest! {
    #[test]
    fn should_encode_decode_rounding_error(v in any::<u64>()) {
        check_roundtrip(&RoundingErrorContainer {
            value: RoundingError::from(v),
        })
        .unwrap();
    }
}

#[derive(Eq, PartialEq, Debug, Decode, Encode)]
struct RoundingErrorContainer {
    #[cbor(n(0), with = "crate::rpc_client::cbor::rounding_error")]
    pub value: RoundingError,
}

pub fn check_roundtrip<T>(v: &T) -> Result<(), TestCaseError>
where
    for<'a> T: PartialEq + std::fmt::Debug + Encode<()> + Decode<'a, ()>,
{
    let mut buf = vec![];
    minicbor::encode(v, &mut buf).expect("encoding should succeed");
    let decoded = minicbor::decode(&buf).expect("decoding should succeed");
    prop_assert_eq!(v, &decoded);
    Ok(())
}
