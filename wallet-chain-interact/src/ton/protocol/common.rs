use super::{block::BlockIdExt, transaction::TransactionId};

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
