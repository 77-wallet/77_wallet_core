use alloy::primitives::hex;
use alloy::signers::k256::ecdsa;
use thiserror::Error;

/// Error thrown by [`Wallet`](crate::Wallet).
#[derive(Debug, Error)]
pub enum WalletError {
    /// [`ecdsa`] error.
    #[error(transparent)]
    Ecdsa(#[from] ecdsa::Error),
    /// [`hex`](mod@hex) error.
    #[error(transparent)]
    Hex(#[from] hex::FromHexError),
    /// [`std::io`] error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// [`coins_bip32`] error.
    #[error(transparent)]
    Bip32(#[from] coins_bip32::Bip32Error),
    /// [`coins_bip39`] error.
    #[error(transparent)]
    Bip39(#[from] coins_bip39::MnemonicError),
    // [`MnemonicBuilder`](super::mnemonic::MnemonicBuilder) error.
    // #[error(transparent)]
    // MnemonicBuilder(#[from] super::mnemonic::MnemonicBuilderError),
    /// [`eth_keystore`] error.
    #[error(transparent)]
    EthKeystore(#[from] crate::crypto::KeystoreError),
    #[error(transparent)]
    Utils(#[from] wallet_utils::Error),
}
