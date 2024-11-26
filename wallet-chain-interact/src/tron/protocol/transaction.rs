use crate::ParseErr;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct BaseTransaction {
    pub amount: u128,
    pub owner_address: String,
    pub to_address: String,
    #[serde(rename = "Permission_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_id: Option<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SendRawTransactionParams {
    #[serde(rename = "txID")]
    pub tx_id: String,
    pub raw_data: String,
    pub raw_data_hex: String,
    pub signature: Vec<String>,
}

impl SendRawTransactionParams {
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        bincode::deserialize::<Self>(bytes)
            .map_err(|e| crate::Error::ParseError(ParseErr::JsonErr(e.to_string())))
    }

    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| crate::Error::ParseError(ParseErr::JsonErr(e.to_string())))
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SendRawTransactionResp {
    pub result: bool,
    #[serde(rename = "txid")]
    pub tx_id: String,
}

// 创建交易的响应
#[derive(Deserialize, Serialize, Debug)]
pub struct CreateTransactionResp<T> {
    pub visible: bool,
    #[serde(rename = "txID")]
    pub tx_id: String,
    pub raw_data: RawData<T>,
    pub raw_data_hex: String,
    #[serde(flatten)]
    ext: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RawData<T> {
    pub contract: Vec<Contract<T>>,
    pub ref_block_bytes: String,
    pub ref_block_hash: String,
    pub expiration: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_limit: Option<u128>,
    pub timestamp: u64,
}

// // 设定一个固定的值(仅做预估本币手续费计算交易字节使用)
// impl From<BaseTransaction> for RawData<BaseTransaction> {
//     fn from(value: BaseTransaction) -> Self {
//         let contract = Contract {
//             parameter: Parameter {
//                 value,
//                 type_url: "type.googleapis.com/protocol.TransferContract".to_string(),
//             },
//             types: "TransferContract".to_string(),
//             permission_id: None,
//         };
//         RawData {
//             contract: vec![contract],
//             ref_block_bytes: "cc46".to_string(),
//             ref_block_hash: "86013f30ec6d034b".to_string(),
//             expiration: 1719569763000,
//             fee_limit: None,
//             timestamp: 1719569703563,
//         }
//     }
// }
// impl RawData<BaseTransaction> {
//     pub(crate) fn raw_data_hex(&self) -> crate::Result<String> {
//         let raw_bytes = bincode::serialize(self)
//             .map_err(|e| crate::Error::ParseError(ParseErr::JsonErr(e.to_string())))?;
//         Ok(hex::encode(raw_bytes))
//     }

//     pub(crate) fn encode_to_hex(&self) -> crate::Result<String> {
//         let raw_bytes = bincode::serialize(self)
//             .map_err(|e| crate::Error::ParseError(ParseErr::JsonErr(e.to_string())))
//             .unwrap();
//         Ok(hex::encode(raw_bytes))
//     }
// }

#[derive(Deserialize, Serialize, Debug)]
pub struct Contract<T> {
    pub parameter: Parameter<T>,
    #[serde(rename = "type")]
    pub types: String,
    #[serde(rename = "Permission_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_id: Option<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Parameter<T> {
    pub value: T,
    pub type_url: String,
}
