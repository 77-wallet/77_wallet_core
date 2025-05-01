// 定位交易的参数
#[derive(Debug, serde::Serialize)]
pub struct LocateTxParams {
    pub source: String,
    pub destination: String,
    pub created_tl: u64,
}
