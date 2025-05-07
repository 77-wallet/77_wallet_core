// 定位交易的参数
#[derive(Debug, serde::Serialize)]
pub struct LocateTxParams {
    pub source: String,
    pub destination: String,
    pub created_tl: u64,
}

// 预估手续费参数
#[derive(Debug, serde::Serialize)]
pub struct EstimateFeeParams {
    pub address: String,
    // msg body
    pub body: String,
    pub init_code: Option<String>,
    pub init_data: Option<String>,
    // 是否验证签名
    pub ignore_chksig: bool,
}
impl EstimateFeeParams {
    pub fn new(address: &str, body: String) -> Self {
        Self {
            address: address.to_owned(),
            body,
            init_code: None,
            init_data: None,
            ignore_chksig: true,
        }
    }
}
