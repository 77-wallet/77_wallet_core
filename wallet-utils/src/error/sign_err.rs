#[derive(Debug, thiserror::Error)]
pub enum SignError {
    #[error("Message error: {0}")]
    Message(String),
    #[error("Key error: {0}")]
    KeyError(String),
}

impl SignError {
    pub fn get_status_code(&self) -> u32 {
        match self {
            SignError::Message(_) => 6400,
            SignError::KeyError(_) => 6401,
        }
    }
}
