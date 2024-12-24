#[derive(Clone)]
pub struct AddressPubkey {
    pub address: String,
    pub pubkey: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenPriceChangeBody {
    pub id: Option<String>,
    // 链码
    pub chain_code: String,
    // 代币编码
    #[serde(rename = "code")]
    pub symbol: String,
    // 默认代币
    pub default_token: Option<bool>,
    // 启用状态
    pub enable: bool,
    // 市值
    pub market_value: f64,
    // 主币
    pub master: bool,
    // 代币名称
    pub name: Option<String>,
    // 单价(usdt)
    pub price: f64,
    // 波动
    pub price_percentage: Option<f64>,
    // 可以状态
    pub status: bool,
    // 代币合约地址
    #[serde(rename = "contractAddress")]
    pub token_address: Option<String>,
    // 24小时交易量
    pub day_change_amount: Option<f64>,
    // 精度
    pub unit: Option<u8>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TokenPopularByPages {
    pub list: Vec<TokenPriceChangeBody>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct NodeData {
    pub node_id: String,
    pub rpc_url: String,
    pub chain_code: String,
}

impl NodeData {
    pub fn new(node_id: &str, rpc_url: &str, chain_code: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            rpc_url: rpc_url.to_string(),
            chain_code: chain_code.to_string(),
        }
    }
}
