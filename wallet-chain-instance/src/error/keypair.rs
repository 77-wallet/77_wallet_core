#[derive(Debug, thiserror::Error)]
pub enum KeypairError {
    #[error("Solana error: `{0}`")]
    Solana(String),
    #[error("Libsecp256k1 error: `{0}`")]
    Libsecp256k1(#[from] libsecp256k1::Error),
}
