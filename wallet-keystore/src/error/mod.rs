pub(crate) mod wallet;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Core error: `{0}`")]
    Core(#[from] wallet_core::Error),
    #[error("Types error: `{0}`")]
    Types(#[from] wallet_types::Error),
    #[error("Tree error: `{0}`")]
    Tree(#[from] wallet_tree::Error),
    #[error("Keystore error: `{0}`")]
    Keystore(#[from] crate::eth_keystore::KeystoreError),
    #[error("Wallet error: `{0}`")]
    PkWallet(#[from] wallet::WalletError),

    #[error("Phrase or salt incorrect")]
    Parase,

    #[error("Tree error: `{0}`")]
    Utils(#[from] wallet_utils::Error),
    #[error("Chain instance error: `{0}`")]
    ChainInstance(#[from] wallet_chain_instance::Error),
}

impl Error {
    pub fn is_network_error(&self) -> bool {
        match self {
            Error::Utils(e) => e.is_network_error(),
            Error::ChainInstance(e) => e.is_network_error(),
            _ => false,
        }
    }
}
