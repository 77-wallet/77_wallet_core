use serde::Deserialize;

use super::transaction::RawTransaction;

#[derive(Debug, Deserialize)]
pub struct MasterChainInfo {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub last: BlockIdExt,
    pub state_root_hash: String,
    pub init: BlockIdExt,
    #[serde(rename = "@extra")]
    pub extra: String,
}

#[derive(Debug, Deserialize)]
pub struct BlockIdExt {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub workchain: i32,
    pub shard: String,
    pub seqno: u32,
    pub root_hash: String,
    pub file_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct BlocksShards {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub shards: Vec<BlockIdExt>,
    #[serde(rename = "@extra")]
    pub extra: String,
}

#[derive(Debug, Deserialize)]
pub struct ConsensusBlock {
    pub consensus_block: u64,
    pub timestamp: f64,
}

#[derive(Debug, Deserialize)]
pub struct BlockTransactionExt<T: std::fmt::Debug> {
    #[serde(rename = "@type")]
    pub type_filed: String,
    pub id: BlockIdExt,
    pub req_count: i64,
    pub incomplete: bool,
    pub transactions: Vec<RawTransaction<T>>,
    #[serde(rename = "@extra")]
    pub extra: String,
}
