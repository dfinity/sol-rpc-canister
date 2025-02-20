use candid::CandidType;
use regex::Regex;
use serde::{Deserialize, Serialize};

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

/// Only log entries matching this filter will be recorded.
#[derive(Clone, Debug, Default, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum LogFilter {
    /// All log entries are recorded.
    #[default]
    ShowAll,
    /// No log entries are recorded.
    HideAll,
    /// Only log entries matching this regular expression are recorded.
    ShowPattern(RegexString),
    /// Only log entries not matching this regular expression are recorded.
    HidePattern(RegexString),
}

impl LogFilter {
    pub fn is_match(&self, message: &str) -> bool {
        match self {
            Self::ShowAll => true,
            Self::HideAll => false,
            Self::ShowPattern(regex) => regex
                .try_is_valid(message)
                .expect("Invalid regex in ShowPattern log filter"),
            Self::HidePattern(regex) => !regex
                .try_is_valid(message)
                .expect("Invalid regex in HidePattern log filter"),
        }
    }
}
