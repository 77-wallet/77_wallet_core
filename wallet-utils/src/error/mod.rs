pub mod crypto;
pub mod http;
pub mod parse;
pub mod serde;
pub mod sign_err;
pub mod snowflake;
pub use snowflake::SnowflakeError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Serde error: {0}")]
    Serde(#[from] serde::SerdeError),
    #[error("Parse error: {0}")]
    Parse(#[from] parse::ParseError),
    #[error("Http error: {0}")]
    Http(#[from] http::HttpError),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Parse error: `{0}`")]
    Sign(#[from] sign_err::SignError),
    #[error("Crypto error: `{0}`")]
    Crypto(#[from] crypto::CryptoError),
    #[error("snowflake error: `{0}`")]
    SnowflakeError(#[from] SnowflakeError),
    #[error("Other error: `{0}`")]
    Other(String),
    #[error("Address index overflow occured")]
    AddressIndexOverflowOccured,
}

impl Error {
    pub fn is_network_error(&self) -> bool {
        matches!(self, Error::Http(_))
    }
}
