use super::{protocol::protobuf::transaction::Raw, provider::Provider};
use async_trait::async_trait;
use contract::ConstantContract;
use protobuf::Message as _;
use std::fmt::Debug;
use wallet_utils::serde_func;

pub mod contract;
pub mod multisig;
pub mod stake;
pub mod transfer;

#[async_trait]
pub trait TronTxOperation<T> {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams>;

    fn get_to(&self) -> String;
}

pub trait TronSimulateOperation<T> {
    fn simulate_raw_transaction(&self) -> crate::Result<RawTransactionParams>;
}

#[async_trait]
pub trait TronConstantOperation<T> {
    async fn constant_contract(&self, provider: &Provider) -> crate::Result<ConstantContract<T>>;
}

// response from tron request
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct TronTransactionResponse<T> {
    pub visible: bool,
    #[serde(rename = "txID")]
    pub tx_id: String,
    pub raw_data: RawData<T>,
    pub raw_data_hex: String,
    #[serde(flatten)]
    ext: Option<serde_json::Value>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RawData<T> {
    pub contract: Vec<Contract<T>>,
    pub ref_block_bytes: String,
    pub ref_block_hash: String,
    pub expiration: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    pub timestamp: u64,
}
impl<T: serde::Serialize> RawData<T> {
    pub fn to_json_string(&self) -> crate::Result<String> {
        Ok(wallet_utils::serde_func::serde_to_string(&self)?)
    }
}

// protobuf raw data
impl Raw {
    pub fn from_str(raw_data_hex: &str) -> crate::Result<Self> {
        Raw::parse_from_bytes(&wallet_utils::hex_func::hex_decode(raw_data_hex)?)
            .map_err(|e| crate::Error::Other(format!("protobuf from str error: {:?}", e)))
    }

    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        self.write_to_bytes()
            .map_err(|e| crate::Error::Other(format!("protobuf to bytes error: {:?}", e)))
    }

    pub fn tx_id(bytes: &[u8]) -> String {
        hex::encode(wallet_utils::sha256(bytes))
    }

    pub fn raw_data_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Contract<T> {
    pub parameter: Parameter<T>,
    #[serde(rename = "type")]
    pub types: String,
    #[serde(rename = "Permission_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_id: Option<u8>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Parameter<T> {
    pub value: T,
    pub type_url: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RawTransactionParams {
    #[serde(rename = "txID")]
    pub tx_id: String,
    pub raw_data: String,
    pub raw_data_hex: String,
    pub signature: Vec<String>,
}

impl std::str::FromStr for RawTransactionParams {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = wallet_utils::base64_to_bytes(s)?;
        Ok(wallet_utils::hex_func::bin_decode_bytes::<Self>(&bytes)?)
    }
}

impl RawTransactionParams {
    pub fn to_string(&self) -> crate::Result<String> {
        let bytes = wallet_utils::hex_func::bin_encode_bytes(self)?;
        Ok(wallet_utils::bytes_to_base64(&bytes))
    }
}

impl<T: serde::Serialize> From<TronTransactionResponse<T>> for RawTransactionParams {
    fn from(value: TronTransactionResponse<T>) -> Self {
        RawTransactionParams {
            tx_id: value.tx_id,
            raw_data: serde_func::serde_to_string(&value.raw_data).unwrap(),
            raw_data_hex: value.raw_data_hex,
            signature: vec![],
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RawTransactionResp {
    pub result: bool,
    #[serde(rename = "txid")]
    pub tx_id: String,
}
