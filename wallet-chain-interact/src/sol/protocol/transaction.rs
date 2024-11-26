use serde::Deserialize;
pub enum CommitmentConfig {
    Processed,
    Confirmed,
    Finalized,
}
impl CommitmentConfig {
    pub fn to_string(&self) -> &'static str {
        match self {
            CommitmentConfig::Processed => "processed",
            CommitmentConfig::Confirmed => "confirmed",
            CommitmentConfig::Finalized => "finalized",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub block_time: u128,
    pub meta: Meta,
    pub slot: u128,
    pub transaction: Transaction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransaction {
    pub meta: Meta,
    pub transaction: Transaction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub compute_units_consumed: u128,
    pub fee: u64,
    pub status: Status,
    // pub log_messages: Vec<String>,
    pub loaded_addresses: LoadedAddresses,
    pub post_balances: Vec<u64>,
    pub post_token_balances: Vec<TokenBalance>,
    pub pre_balances: Vec<u64>,
    pub pre_token_balances: Vec<TokenBalance>,
}
#[derive(Debug, Deserialize)]
pub struct LoadedAddresses {
    pub writable: Vec<String>,
    pub readonly: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub account_index: u64,
    pub mint: String,
    pub owner: String,
    pub program_id: String,
    pub ui_token_amount: UiTokenAmount,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiTokenAmount {
    pub amount: String,
    pub decimals: u64,
    // pub ui_amount: Option<f64>,
    // pub ui_amount_string: String,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Status {
    Ok(Option<String>),
    Err(Option<serde_json::Value>),
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    pub message: Message,
    pub signatures: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub account_keys: Vec<String>,
    // pub address_table_lookups: Vec<AddressTableLookups>,
    // pub header: Vec<String>,
    // pub instructions: Vec<TxInstruction>,
    pub recent_blockhash: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressTableLookups {
    pub account_key: Vec<i32>,
    pub readonly_indexes: Vec<i32>,
    pub writable_indexes: Vec<i32>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignatureStatus {
    pub slot: u64,
    pub confirmations: Option<i64>,
    pub confirmation_status: String,
    pub status: Status,
}
