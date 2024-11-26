use super::transaction::BlockTransaction;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BlockHash {
    pub blockhash: String,
    #[serde(rename = "lastValidBlockHeight")]
    pub last_valid_block_height: u128,
}
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub block_height: u64,
    pub block_time: Option<u64>,
    pub blockhash: String,
    pub parent_slot: u64,
    pub previous_blockhash: String,
    pub transactions: Vec<BlockTransaction>,
}
