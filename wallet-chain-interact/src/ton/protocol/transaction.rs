use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawTransaction {
    #[serde(rename = "@type")]
    pub type_field: String,

    pub utime: u64,
    pub data: String,
    pub transaction_id: TransactionId,
    pub fee: String,
    pub storage_fee: String,
    pub other_fee: String,
    pub in_msg: RawMessage,
    pub out_msgs: Vec<RawMessage>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionId {
    #[serde(rename = "@type")]
    pub type_field: String,

    pub lt: String,
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct RawMessage {
    #[serde(rename = "@type")]
    pub type_field: String,

    pub source: String,
    pub destination: String,
    pub value: String,
    pub fwd_fee: String,
    pub ihr_fee: String,
    pub created_lt: String,
    pub body_hash: String,
    pub msg_data: MsgData,

    // `message` 字段是选填（可能在某些消息中缺失）
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "@type")]
pub enum MsgData {
    #[serde(rename = "msg.dataRaw")]
    Raw { body: String, init_state: String },
}

#[derive(Debug, Deserialize)]
pub struct SendBocReturn {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub hash: String,
    #[serde(rename = "@extra")]
    pub extra: String,
}
