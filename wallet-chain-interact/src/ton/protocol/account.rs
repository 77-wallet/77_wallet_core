use super::{block::BlockIdExt, transaction::TransactionId};

#[derive(Debug, serde::Deserialize)]
pub struct AddressInformation {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub balance: String,
    pub code: String,
    pub data: String,
    pub last_transaction_id: TransactionId,
    pub block_id: BlockIdExt,
    pub frozen_hash: String,
    pub sync_utime: u64,
    #[serde(rename = "@extra")]
    pub extra: String,
    pub state: String,
}
