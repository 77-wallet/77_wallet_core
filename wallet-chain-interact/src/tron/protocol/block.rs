use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
pub struct TronBlock {
    #[serde(rename = "blockID")]
    pub block_id: String,
    pub block_header: TronBlokHeader,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct TronBlokHeader {
    pub raw_data: TronRawData,
    pub witness_signature: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct TronRawData {
    pub number: u64,
    #[serde(rename = "txTrieRoot")]
    pub tx_trie_root: String,
    pub witness_address: String,
    #[serde(rename = "parentHash")]
    pub parent_hash: String,
    pub version: u64,
    pub timestamp: u64,
}
