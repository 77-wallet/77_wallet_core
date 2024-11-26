use serde::{Deserialize, Serialize};

use crate::BillResourceConsume;
#[derive(Deserialize, Serialize, Debug)]
pub struct TransactionInfo {
    pub id: String,
    #[serde(default)]
    pub fee: f64,
    #[serde(rename = "blockNumber")]
    pub block_number: u128,
    #[serde(rename = "blockTimeStamp")]
    pub block_timestamp: u128,
    #[serde(rename = "contractResult")]
    pub contract_result: Vec<String>,
    pub receipt: TronReceipt,
    pub result: Option<String>,
    // if the transaction is failed, this field will be filled
    #[serde(rename = "resMessage")]
    pub res_message: Option<String>,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct TronReceipt {
    pub net_usage: Option<u64>,
    pub energy_usage: Option<u64>,
    pub energy_usage_total: Option<u64>,
}

impl TronReceipt {
    pub fn get_bill_resource_consumer(&self) -> BillResourceConsume {
        BillResourceConsume::new_tron(
            self.net_usage.unwrap_or_default(),
            self.energy_usage.unwrap_or_default(),
        )
    }
}
