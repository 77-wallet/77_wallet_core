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

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prioritization(pub Vec<PrioritizationFee>);

impl Prioritization {
    pub fn get_avg(&self) -> u64 {
        if self.0.len() == 0 {
            return 0;
        }

        let last_records = if self.0.len() > 20 {
            &self.0[self.0.len() - 20..]
        } else {
            &self.0[..]
        };

        let amount = last_records
            .iter()
            .map(|i| i.prioritization_fee)
            .sum::<u64>();

        amount / self.0.len() as u64
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrioritizationFee {
    pub slot: u64,
    pub prioritization_fee: u64,
}
