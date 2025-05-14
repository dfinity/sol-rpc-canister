use minicbor::{
    decode::Decoder,
    encode::{Encoder, Write},
};
use sol_rpc_types::RoundingError;

pub fn decode<Ctx>(
    d: &mut Decoder<'_>,
    _ctx: &mut Ctx,
) -> Result<RoundingError, minicbor::decode::Error> {
    d.u64().map(RoundingError::from)
}

pub fn encode<Ctx, W: Write>(
    v: &RoundingError,
    e: &mut Encoder<W>,
    _ctx: &mut Ctx,
) -> Result<(), minicbor::encode::Error<W::Error>> {
    e.u64(*v.as_ref())?.ok()
}
