#[derive(serde::Deserialize, Debug)]
pub struct FeeRate {
    #[serde(rename = "feerate")]
    pub fee_rate: f64,
    pub blocks: u64,
}
