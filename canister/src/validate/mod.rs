#[cfg(test)]
mod tests;

use crate::constants::{API_KEY_MAX_SIZE, VALID_API_KEY_CHARS};
use const_format::formatcp;

const API_KEY_TOO_LONG_ERROR_MESSAGE: &str =
    formatcp!("API key must be <= {} bytes", API_KEY_MAX_SIZE);

pub fn validate_api_key(api_key: &str) -> Result<(), &'static str> {
    if api_key.is_empty() {
        Err("API key must not be an empty string")
    } else if api_key.len() > API_KEY_MAX_SIZE {
        Err(API_KEY_TOO_LONG_ERROR_MESSAGE)
    } else if api_key
        .chars()
        .any(|char| !VALID_API_KEY_CHARS.contains(char))
    {
        Err("Invalid character in API key")
    } else {
        Ok(())
    }
}
