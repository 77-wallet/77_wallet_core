use thiserror::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum ParseErr {
    #[error("address parse error {0}")]
    AddressPraseErr(String),
    #[error("rpc url error {0}")]
    RpcUrlPraseErr(String),
    #[error("value convert error {0}")]
    ValueErr(String),
    #[error("tx hash error")]
    TxHashErr,
    #[error("json serialize{0}")]
    JsonErr(String),
    #[error("serde deserialize{0}")]
    SerdeErr(#[from] serde_json::Error),
    #[error("serialize {0}")]
    Serialize(String),
}

#[derive(Error, Debug)]
pub enum UtxoError {
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Insufficient fee")]
    InsufficientFee,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    TransportError(#[from] wallet_transport::errors::TransportError),
    #[error("rpc node return error: {0}")]
    RpcNode(String),
    #[error("utils error {0}")]
    UtilsError(#[from] wallet_utils::error::Error),
    #[error("parse error {0}")]
    AbiParseError(String),
    #[error("types error {0}")]
    Types(#[from] wallet_types::Error),
    // flow to optimize
    #[error("hex error {0}")]
    HexError(String),
    #[error("btc script error {0}")]
    BtcScript(String),
    #[error("sign error {0}")]
    SignError(String),
    #[error("{0}")]
    Other(String),
    #[error("not support api:{0}")]
    NotSupportApi(String),
    #[error("rpc error {0}")]
    RpcError(String),
    #[error("parse error {0}")]
    ParseError(#[from] ParseErr),
    #[error("utxo error")]
    UtxoError(#[from] UtxoError),
    #[error("transfer error {0}")]
    TransferError(String),
}

impl Error {
    pub fn is_network_error(&self) -> bool {
        match self {
            Error::TransportError(e) => e.is_network_error(),
            Error::UtilsError(e) => e.is_network_error(),
            _ => false,
        }
    }
}
