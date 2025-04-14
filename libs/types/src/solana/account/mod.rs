use candid::{CandidType, Deserialize};
use serde::Serialize;
use solana_account_decoder_client_types::UiAccountEncoding;

/// Solana [account](https://solana.com/docs/references/terminology#account) information.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct AccountInfo {
    /// Number of lamports assigned to this account.
    pub lamports: u64,
    /// Data associated with the account.
    pub data: AccountData,
    /// base-58 encoded Pubkey of the program this account has been assigned to.
    pub owner: String,
    /// Boolean indicating if the account contains a program (and is strictly read-only).
    pub executable: bool,
    /// The epoch at which this account will next owe rent.
    #[serde(rename = "rentEpoch")]
    pub rent_epoch: u64,
    /// The data size of the account.
    pub space: u64,
}

impl From<solana_account_decoder_client_types::UiAccount> for AccountInfo {
    fn from(account: solana_account_decoder_client_types::UiAccount) -> Self {
        AccountInfo {
            lamports: account.lamports,
            data: account.data.into(),
            owner: account.owner,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
            // The `space` field is optional for backwards compatibility reasons, however it should
            // always contain a value.
            space: account.space.expect("'space' field should not be null"),
        }
    }
}

impl From<AccountInfo> for solana_account_decoder_client_types::UiAccount {
    fn from(account: AccountInfo) -> Self {
        solana_account_decoder_client_types::UiAccount {
            lamports: account.lamports,
            data: account.data.into(),
            owner: account.owner,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
            space: Some(account.space),
        }
    }
}

/// Represents the data stored in a Solana [account](https://solana.com/docs/references/terminology#account).
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum AccountData {
    /// The data is formatted as a binary string. This is a legacy format retained for RPC backwards compatibility
    #[serde(rename = "legacyBinary")]
    LegacyBinary(String),
    /// The data is formatted as a JSON [`ParsedAccount`].
    #[serde(rename = "json")]
    Json(ParsedAccount),
    /// The data is formatted as a string containing the account data encoded according to one of
    /// the [`AccountEncoding`] formats.
    #[serde(rename = "binary")]
    Binary(String, AccountEncoding),
}

impl From<solana_account_decoder_client_types::UiAccountData> for AccountData {
    fn from(data: solana_account_decoder_client_types::UiAccountData) -> Self {
        use solana_account_decoder_client_types::UiAccountData;
        match data {
            UiAccountData::LegacyBinary(value) => Self::LegacyBinary(value),
            UiAccountData::Json(value) => Self::Json(value.into()),
            UiAccountData::Binary(value, encoding) => Self::Binary(value, encoding.into()),
        }
    }
}

impl From<AccountData> for solana_account_decoder_client_types::UiAccountData {
    fn from(data: AccountData) -> Self {
        use solana_account_decoder_client_types::UiAccountData;
        match data {
            AccountData::LegacyBinary(value) => UiAccountData::LegacyBinary(value),
            AccountData::Json(value) => UiAccountData::Json(value.into()),
            AccountData::Binary(value, encoding) => UiAccountData::Binary(value, encoding.into()),
        }
    }
}

/// Represents parsed Solana [account](https://solana.com/docs/references/terminology#account) data.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct ParsedAccount {
    /// The Solana [program](https://solana.com/docs/references/terminology#program) that interprets the data.
    pub program: String,
    /// The account data parsed a JSON and formatted as a string.
    pub parsed: String,
    /// The data size of the account.
    pub space: u64,
}

impl From<solana_account_decoder_client_types::ParsedAccount> for ParsedAccount {
    fn from(account: solana_account_decoder_client_types::ParsedAccount) -> Self {
        Self {
            program: account.program,
            parsed: serde_json::to_string(&account.parsed)
                .expect("Unable to convert JSON to string"),
            space: account.space,
        }
    }
}

impl From<ParsedAccount> for solana_account_decoder_client_types::ParsedAccount {
    fn from(account: ParsedAccount) -> Self {
        Self {
            program: account.program,
            parsed: serde_json::from_str(&account.parsed).expect("Unable to parse string as JSON"),
            space: account.space,
        }
    }
}

/// Represents an encoding format for Solana [account](https://solana.com/docs/references/terminology#account) data.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum AccountEncoding {
    /// The account data is formatted as a binary string.
    #[serde(rename = "binary")]
    Binary, // Legacy. Retained for RPC backwards compatibility
    /// The account data is formatted as a base-58 string.
    #[serde(rename = "base58")]
    Base58,
    /// The account data is formatted as a base-64 string.
    #[serde(rename = "base64")]
    Base64,
    /// The account data was first compressed using [Zstandard](http://facebook.github.io/zstd/) and the
    /// result was then formatted as a base-64 string.
    #[serde(rename = "base64+zstd")]
    Base64Zstd,
    /// The account data is formatted as a JSON string.
    #[serde(rename = "jsonParsed")]
    JsonParsed,
}

impl From<UiAccountEncoding> for AccountEncoding {
    fn from(encoding: UiAccountEncoding) -> Self {
        use solana_account_decoder_client_types::UiAccountEncoding;
        match encoding {
            UiAccountEncoding::Binary => Self::Binary,
            UiAccountEncoding::Base58 => Self::Base58,
            UiAccountEncoding::Base64 => Self::Base64,
            UiAccountEncoding::JsonParsed => Self::JsonParsed,
            UiAccountEncoding::Base64Zstd => Self::Base64Zstd,
        }
    }
}

impl From<AccountEncoding> for UiAccountEncoding {
    fn from(encoding: AccountEncoding) -> Self {
        match encoding {
            AccountEncoding::Binary => Self::Binary,
            AccountEncoding::Base58 => Self::Base58,
            AccountEncoding::Base64 => Self::Base64,
            AccountEncoding::JsonParsed => Self::JsonParsed,
            AccountEncoding::Base64Zstd => Self::Base64Zstd,
        }
    }
}
