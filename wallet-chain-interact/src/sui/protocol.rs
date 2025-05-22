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
