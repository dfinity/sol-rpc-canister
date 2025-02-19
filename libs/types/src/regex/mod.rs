use candid::{CandidType, Deserialize};
use regex::Regex;
use serde::Serialize;

/// A string used as a regex pattern.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct RegexString(pub String);

impl From<&str> for RegexString {
    fn from(value: &str) -> Self {
        RegexString(value.to_string())
    }
}

impl RegexString {
    /// Compile the string into a regular expression.
    ///
    /// This is a relatively expensive operation that's currently not cached.
    pub fn compile(&self) -> Result<Regex, regex::Error> {
        Regex::new(&self.0)
    }

    /// Checks if the given string matches the compiled regex pattern.
    ///
    /// Returns `Ok(true)` if `value` matches, `Ok(false)` if not, or an error if the regex is invalid.
    pub fn try_is_valid(&self, value: &str) -> Result<bool, regex::Error> {
        Ok(self.compile()?.is_match(value))
    }
}

/// A regex-based substitution with a pattern and replacement string.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct RegexSubstitution {
    /// The pattern to be matched.
    pub pattern: RegexString,
    /// The string to replace occurrences [`pattern`] with.
    pub replacement: String,
}
