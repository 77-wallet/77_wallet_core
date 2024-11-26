use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct TotalSupply {
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: f64,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}
