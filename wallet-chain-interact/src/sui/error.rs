use thiserror::Error;

#[derive(Error, Debug)]
pub enum SuiError {
    #[error("Move error: {0}")]
    MoveError(String),
    // #[error("sui sdk error {0}")]
    // SuiSdk(#[from] sui_sdk::error::Error),
    #[error("Insufficient balance: only {0} available, but {1} needed")]
    InsufficientBalance(u64, u64),
}
