use sui_json_rpc_types::Coin;

use super::consts::SUI_VALUE;

/// Checkpoint 详情
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointResult {
    pub epoch: String,
    pub sequence_number: String,
    pub digest: String,
    pub network_total_transactions: String,
    pub previous_digest: String,
    pub epoch_rolling_gas_cost_summary: GasCostSummary,
    pub timestamp_ms: String,
    pub transactions: Vec<String>,
    pub checkpoint_commitments: Vec<String>,
    pub validator_signature: String,
}

/// Gas 费用汇总
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GasCostSummary {
    pub computation_cost: String,
    pub storage_cost: String,
    pub storage_rebate: String,
    pub non_refundable_storage_fee: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct EstimateFeeResp {
    pub gas_fee: u64,
    pub gas_price: u64,
}

impl EstimateFeeResp {
    pub fn new(gas_fee: u64, gas_price: u64) -> Self {
        Self { gas_fee, gas_price }
    }

    pub fn get_fee(&self) -> u64 {
        self.gas_fee + (self.gas_fee as f64 * 0.5) as u64
    }

    pub fn get_fee_f64(&self) -> f64 {
        self.get_fee() as f64 / SUI_VALUE
    }
}

pub struct GasObject {
    pub coin: Vec<Coin>,
}
impl GasObject {
    pub fn new(coin: Vec<Coin>) -> Self {
        Self { coin }
    }
}
