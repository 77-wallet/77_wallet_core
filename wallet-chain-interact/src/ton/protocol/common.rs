use super::{block::BlockIdExt, transaction::TransactionId};
use crate::ton::errors::TonError;
use tonlib_core::cell::BagOfCells;

#[derive(Debug, serde::Serialize)]
pub struct RunGetMethodParams<T> {
    pub address: String,
    pub method: String,
    pub stack: Vec<T>,
    // pub seqno: u32,
}
impl<T> RunGetMethodParams<T> {
    pub fn new(address: &str, method: &str, stack: Vec<T>) -> Self {
        Self {
            address: address.to_string(),
            method: method.to_string(),
            stack,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct RunGetMethodResp {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub gas_used: u64,
    pub stack: Vec<StackItem>,
    pub exit_code: i32,
    #[serde(rename = "@extra")]
    pub extra: String,
    pub block_id: BlockIdExt,
    pub last_transaction_id: TransactionId,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum StackItem {
    Num(String, String),
    Slice(String, SliceContent),
}

#[derive(Debug, serde::Deserialize)]
pub struct SliceContent {
    pub bytes: String,
    pub object: CellObject,
}

#[derive(Debug, serde::Deserialize)]
pub struct CellObject {
    pub data: CellData,
    pub refs: Vec<serde_json::Value>,
    pub special: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct CellData {
    pub b64: String,
    pub len: usize,
}

#[derive(Debug, serde::Deserialize)]
pub struct ConfigParams {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub config: Config,
}
impl ConfigParams {
    pub fn parse_gas_price(&self) -> Result<GasConfig, TonError> {
        let bag = BagOfCells::parse_base64(&self.config.bytes)?.single_root()?;

        let mut parse = bag.parser();
        // 标识
        let _t = parse.load_u8(8)?;

        let special = parse.load_u64(64)?;
        let flat_gas_limit = parse.load_u64(64)?;

        let _t = parse.load_u8(8)?;

        let res = GasConfig {
            special_gas_limit: special,
            flat_gas_limit,
            flat_gas_price: parse.load_u64(64)?,
            gas_price: parse.load_u64(64)?,
            gas_limit: parse.load_u64(64)?,
            gas_credit: parse.load_u64(64)?,
            block_gas_limit: parse.load_u64(64)?,
            freeze_due_limit: parse.load_u64(64)?,
            delete_due_limit: parse.load_u64(64)?,
        };

        Ok(res)
    }
}

#[derive(Debug, Clone)]
pub struct GasConfig {
    pub special_gas_limit: u64,
    pub flat_gas_limit: u64,
    pub flat_gas_price: u64,
    pub gas_price: u64,
    pub gas_limit: u64,
    pub gas_credit: u64,
    pub block_gas_limit: u64,
    pub freeze_due_limit: u64,
    pub delete_due_limit: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub bytes: String,
}
