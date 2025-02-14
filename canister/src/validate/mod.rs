#[cfg(test)]
mod tests;

use crate::constants::VALID_API_KEY_CHARS;

pub fn validate_api_key(api_key: &str) -> Result<(), &'static str> {
    if api_key.is_empty() {
        Err("API key must not be an empty string")
    } else if api_key.len() > 200 {
        Err("API key must be <= 200 characters")
    } else if api_key
        .chars()
        .any(|char| !VALID_API_KEY_CHARS.contains(char))
    {
        Err("Invalid character in API key")
    } else {
        Ok(())
    }
}
