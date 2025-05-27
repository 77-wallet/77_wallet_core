use thiserror::Error;

#[derive(Error, Debug)]
pub enum SuiError {
    #[error("Move error: {0}")]
    MoveError(String),
    #[error("Insufficient balance: only {0} available, but {1} needed")]
    InsufficientBalance(u64, u64),
    #[error("estimate gas error: {0}")]
    GasError(String),
    #[error("Insufficient fee: only {0} available, but {1} needed")]
    InsufficientFee(u64, u64),
}
