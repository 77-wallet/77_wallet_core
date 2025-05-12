// 定位交易的参数
#[derive(Debug, serde::Serialize)]
pub struct LocateTxParams {
    pub source: String,
    pub destination: String,
    pub created_lt: u64,
}

// 预估手续费参数
#[derive(Debug, serde::Serialize)]
pub struct EstimateFeeParams {
    pub address: String,
    pub body: String,
    pub init_code: Option<String>,
    pub init_data: Option<String>,
    pub ignore_chksig: bool,
}
impl EstimateFeeParams {
    pub fn new(address: &str, body: String, ignore_chksig: bool) -> Self {
        Self {
            address: address.to_owned(),
            body,
            init_code: None,
            init_data: None,
            ignore_chksig,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct QueryTransParams {
    pub address: String,
    pub limit: Option<i32>,
    pub lt: Option<u64>,
    pub hash: Option<String>,
}
impl QueryTransParams {
    pub fn new_with_limit(address: &str, limit: i32) -> Self {
        Self {
            address: address.to_owned(),
            limit: Some(limit),
            lt: None,
            hash: None,
        }
    }
}
