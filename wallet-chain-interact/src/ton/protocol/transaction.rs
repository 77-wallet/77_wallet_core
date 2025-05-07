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

#[derive(Debug, Deserialize)]
pub struct EstimateFeeResp {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub source_fees: SourceFees,
    pub destination_fees: Vec<DestinationFees>,
    #[serde(rename = "@extra")]
    pub extra: String,
}

impl EstimateFeeResp {
    pub fn get_fee(&self) -> u64 {
        self.source_fees.in_fwd_fee
            + self.source_fees.storage_fee
            + self.source_fees.gas_fee
            + self.source_fees.fwd_fee
    }
}

#[derive(Debug, Deserialize)]
pub struct SourceFees {
    #[serde(rename = "@type")]
    pub type_field: String,

    pub in_fwd_fee: u64,
    pub storage_fee: u64,
    pub gas_fee: u64,
    pub fwd_fee: u64,
}

#[derive(Debug, Deserialize)]
pub struct DestinationFees {
    // 如果 destination_fees 不为空，你可以根据实际字段结构补充此结构体
}
