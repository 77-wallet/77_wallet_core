use crate::ton::{consts::TON_VALUE, errors::TonError};
use serde::Deserialize;
use tonlib_core::{
    cell::BagOfCells,
    message::{JettonTransferMessage, TonMessage},
    tlb_types::traits::TLBObject,
};
pub trait GetAddress {
    fn get_address(&self) -> String;
}
impl GetAddress for String {
    fn get_address(&self) -> String {
        self.clone()
    }
}

impl GetAddress for AddressId {
    fn get_address(&self) -> String {
        self.account_address.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct RawTransaction<T: std::fmt::Debug = String> {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub utime: u64,
    pub data: String,
    pub transaction_id: TransactionId,
    pub fee: String,
    pub storage_fee: String,
    pub other_fee: String,
    pub in_msg: RawMessage<T>,
    pub out_msgs: Vec<RawMessage<T>>,
}

impl<T: std::fmt::Debug> RawTransaction<T> {
    pub fn get_fee(&self) -> crate::Result<f64> {
        let fee = wallet_utils::unit::str_to_num::<f64>(&self.fee)?;
        // let s_fee = wallet_utils::unit::str_to_num::<f64>(&self.storage_fee)?;
        // let o_fee = wallet_utils::unit::str_to_num::<f64>(&self.other_fee)?;

        Ok(fee / TON_VALUE as f64)
    }
}

#[derive(Debug, Deserialize)]
pub struct TransactionId {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub lt: String,
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct RawMessage<T: std::fmt::Debug> {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub hash: String,
    pub source: T,
    pub destination: T,
    pub value: String,
    pub fwd_fee: String,
    pub ihr_fee: String,
    pub created_lt: String,
    pub body_hash: String,
    pub msg_data: MsgData,
    pub message: Option<String>,
}

impl<T: std::fmt::Debug> RawMessage<T> {
    // 简单验证是否是token交易:目前根据操作码进行判断的,后续估计需要加入地址类型
    pub fn is_token(&self) -> crate::Result<Option<bool>> {
        match &self.msg_data {
            MsgData::Raw {
                body,
                init_state: _,
            } => {
                let bag = BagOfCells::parse_base64(&body).map_err(TonError::CellBuild)?;
                let cell = bag
                    .single_root()
                    .map_err(TonError::CellBuild)?
                    .to_cell()
                    .map_err(TonError::CellBuild)?;

                if cell.data().is_empty() {
                    Ok(Some(false))
                } else {
                    let mut parse = cell.parser();

                    let op_code = parse.load_u32(32).map_err(TonError::CellBuild)?;

                    Ok(Some(op_code == 0x0f8a7ea5))
                }
            }
            MsgData::Text { text: _ } => Ok(Some(false)),
        }
    }

    pub fn parse_token_transfer(&self) -> crate::Result<JettonTransferMessage> {
        match &self.msg_data {
            MsgData::Raw {
                body,
                init_state: _,
            } => {
                let bag = BagOfCells::parse_base64(&body).map_err(TonError::CellBuild)?;
                let cell = bag
                    .single_root()
                    .map_err(TonError::CellBuild)?
                    .to_cell()
                    .map_err(TonError::CellBuild)?;

                Ok(JettonTransferMessage::parse(&cell).map_err(TonError::TonMsg)?)
            }
            MsgData::Text { text: _ } => {
                Err(TonError::NotTokenParse("text raw_data ".to_string()))?
            }
        }
    }

    pub fn value(&self) -> crate::Result<u128> {
        Ok(wallet_utils::unit::str_to_num::<u128>(&self.value)?)
    }
}

impl<T: GetAddress + std::fmt::Debug> RawMessage<T> {
    pub fn get_from(&self) -> String {
        self.source.get_address()
    }
    pub fn get_to(&self) -> String {
        self.destination.get_address()
    }
}

#[derive(Debug, Deserialize)]
pub struct AddressId {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub account_address: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "@type")]
pub enum MsgData {
    #[serde(rename = "msg.dataRaw")]
    Raw { body: String, init_state: String },
    #[serde(rename = "msg.dataText")]
    Text { text: String },
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

    pub fn get_fee_ton(&self) -> f64 {
        self.get_fee() as f64 / TON_VALUE as f64
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
