#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("BlockPadding error: {0}")]
    BlockPadding(String),
    #[error("InoutPad error: {0}")]
    InoutPad(String),
    #[error("Base64 error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("InvalidLength error: {0}")]
    InvalidLength(#[from] aes::cipher::InvalidLength),
}

impl CryptoError {
    pub fn get_status_code(&self) -> u32 {
        match self {
            CryptoError::BlockPadding(_) => 6400,
            CryptoError::InoutPad(_) => 6400,
            CryptoError::Base64(_) => 6401,
            CryptoError::InvalidLength(_) => 6401,
        }
    }
}

impl From<aes::cipher::block_padding::UnpadError> for CryptoError {
    fn from(value: aes::cipher::block_padding::UnpadError) -> Self {
        Self::BlockPadding(value.to_string())
    }
}
impl From<aes::cipher::inout::PadError> for CryptoError {
    fn from(value: aes::cipher::inout::PadError) -> Self {
        Self::InoutPad(value.to_string())
    }
}
