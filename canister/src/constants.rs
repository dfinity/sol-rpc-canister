// The default value of max_response_bytes is 2_000_000.
pub const DEFAULT_MAX_RESPONSE_BYTES: u64 = 2_000_000;

// Cycles (per node) which must be passed with each RPC request
// as processing fee.
pub const COLLATERAL_CYCLES_PER_NODE: u128 = 10_000_000;

pub const CONTENT_TYPE_HEADER_LOWERCASE: &str = "content-type";
pub const CONTENT_TYPE_VALUE: &str = "application/json";

pub const API_KEY_REPLACE_STRING: &str = "{API_KEY}";
pub const API_KEY_MAX_SIZE: usize = 512;
pub const VALID_API_KEY_CHARS: &str =
    "0123456789ABCDEFGHIJKLMNOPQRTSUVWXYZabcdefghijklmnopqrstuvwxyz$-_.+!*";
