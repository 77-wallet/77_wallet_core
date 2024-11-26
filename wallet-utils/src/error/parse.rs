#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("FromStringUtf8 error: {0}")]
    FromStringUtf8(#[from] std::string::FromUtf8Error),
    #[error("FromStrUtf8 error: {0}")]
    FromStrUtf8(#[from] std::str::Utf8Error),
    #[error("FromHex error: {0}")]
    FromHex(String),
    #[error("FromHex error: {0}")]
    FromConstHex(#[from] alloy::hex::FromHexError),
    #[error("Method parse error: {0}")]
    HttpMethod(#[from] http::method::InvalidMethod),
    #[error("ToInt parse error: {0}")]
    ToInt(#[from] std::num::ParseIntError),
    #[error("ToFloat parse error: {0}")]
    ToFloat(#[from] std::num::ParseFloatError),
    #[error("Parse http body to bytes failed")]
    HttpBodyToBytesFailed,
    #[error("Parse Decimal to int 64 failed")]
    DecimalToI64Failed,
    #[error("Parse Decimal to float 64 failed")]
    DecimalToF64Failed,
    #[error("From float 64 to Decimal failed")]
    FromF64ToDecimalFailed,
    #[error("Parse vector to array failed")]
    VecToArrayFailed,
    #[error("Address error: {0}")]
    AddressError(#[from] alloy::primitives::AddressError),
    #[error("Solana signature error: {0}")]
    SolanaSignatureError(String),
    #[error("address convert failed: {0}")]
    AddressConvertFailed(String),
    #[error("Decimal error: {0}")]
    Decimal(#[from] rust_decimal::Error),
    #[error("Bech32 hrp error: {0}")]
    Bech32Hrp(#[from] bech32::primitives::hrp::Error),
    #[error("Custom enum error: {0}")]
    CustomEnum(String),
    #[error("unit convert failed{0}")]
    UnitConvertFailed(String),
}

impl ParseError {
    pub fn get_status_code(&self) -> u32 {
        match self {
            ParseError::FromStringUtf8(_) => 6300,
            ParseError::FromStrUtf8(_) => 6300,
            ParseError::FromHex(_) => 6301,
            ParseError::FromConstHex(_) => 6301,
            ParseError::HttpMethod(_) => 6301,
            ParseError::ToInt(_) => 6304,
            ParseError::ToFloat(_) => 6304,
            ParseError::HttpBodyToBytesFailed => 6309,
            ParseError::DecimalToI64Failed => 6310,
            ParseError::DecimalToF64Failed => 6310,
            ParseError::FromF64ToDecimalFailed => 6310,
            ParseError::VecToArrayFailed => 6311,
            ParseError::AddressError(_) => 6311,
            ParseError::AddressConvertFailed(_) => 6311,
            ParseError::Decimal(_) => 6311,
            ParseError::Bech32Hrp(_) => 6311,
            ParseError::CustomEnum(_) => 6311,
            ParseError::UnitConvertFailed(_) => 6311,
            ParseError::SolanaSignatureError(_) => 6311,
        }
    }
}
