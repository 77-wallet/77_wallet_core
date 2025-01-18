pub mod keypair;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("This btc address type can't generate derivation path")]
    BtcAddressTypeCantGenDerivationPath,
    #[error("Core error: {0}")]
    Core(#[from] wallet_core::Error),
    #[error("Utils error: {0}")]
    Utils(#[from] wallet_utils::Error),
    #[error("Keypair error: `{0}`")]
    Keypair(#[from] keypair::KeypairError),
    #[error("Types error: `{0}`")]
    Types(#[from] wallet_types::Error),
    #[error("Bech32 segwit encode error: `{0}`")]
    Bech32SegwitEncode(#[from] bech32::segwit::EncodeError),
    #[error("Invalid public key length")]
    InvalidPublicKeyLength, // #[error("Net failed: {0:?}")]
    #[error("Bitcoin bip32 error: `{0}`")]
    BitcoinBip32(#[from] bitcoin::bip32::Error),
    #[error("Secp256k1 Out of range error: `{0}`")]
    Secp256k1OutOfRange(#[from] secp256k1::scalar::OutOfRangeError),
    #[error("Secp256k1 Out of range error: `{0}`")]
    Secp256k1(#[from] secp256k1::Error),
    #[error("HdPath error: `{0}`")]
    HdPath(String),
    #[error("parase private key error:: `{0}`")]
    ParasePrivateKey(String),
}

impl Error {
    pub fn is_network_error(&self) -> bool {
        match self {
            Error::Utils(e) => e.is_network_error(),
            _ => false,
        }
    }
}

impl From<hdpath::Error> for Error {
    fn from(value: hdpath::Error) -> Self {
        match value {
            hdpath::Error::HighBitIsSet => Error::HdPath("HighBitIsSet".to_string()),
            hdpath::Error::InvalidLength(len) => Error::HdPath(format!("InvalidLength({})", len)),
            hdpath::Error::InvalidPurpose(pur) => Error::HdPath(format!("InvalidPurpose({})", pur)),
            hdpath::Error::InvalidStructure => Error::HdPath("InvalidStructure".to_string()),
            hdpath::Error::InvalidFormat => Error::HdPath("InvalidFormat".to_string()),
        }
    }
}
