use thiserror::Error;
use tonlib_core::{TonAddressParseError, cell::TonCellError, message::TonMessageError};

#[derive(Error, Debug)]
pub enum TonError {
    #[error("run getMethod Resp:{0}")]
    RunGetMethodResp(String),
    #[error("cell build {0}")]
    CellBuild(#[from] TonCellError),
    #[error("ton address {0}")]
    TonAddress(#[from] TonAddressParseError),
    #[error("ton message error {0}")]
    TonMsg(#[from] TonMessageError),
    #[error("{0}")]
    TonNodeError(#[from] wallet_transport::errors::TransportError),
    #[error("{0}")]
    NotTokenParse(String),
}
