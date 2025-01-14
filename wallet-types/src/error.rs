#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum Error {
    #[error("HdPath error")]
    HdPath,
    #[error("Unknown chain code")]
    UnknownChainCode,
    #[error("Unknown coin type: {0}")]
    UnknownCoinType(u32),
    #[error("Btc need address type")]
    BtcNeedAddressType,
    #[error("Ltc need address type")]
    LtcNeedAddressType,
    #[error("Invalid BtcAddressType: {0}")]
    BtcAddressTypeInvalid(String),
    #[error("Invalid BtcAddressCategory: {0}")]
    BtcAddressCategoryInvalid(String),
    #[error("Invalid LtcAddressCategory: {0}")]
    LtcAddressCategoryInvalid(String),
}
