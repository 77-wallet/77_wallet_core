use crate::ltc::{consts::LTC_DECIMAL, utxos::Utxo};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use wallet_types::chain::address::r#type::LtcAddressType;
use wallet_utils::unit;

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub txid: String,
    pub hash: String,
    pub version: u32,
    pub size: u32,
    pub vsize: u32,
    pub weight: u64,
    pub locktime: u32,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    pub hex: String,
    pub blockhash: String,
    pub confirmations: u32,
    pub time: u32,
    pub blocktime: u32,
}

impl Transaction {
    pub fn total_vout(&self) -> f64 {
        self.vout.iter().map(|v| v.value).sum()
    }

    //  获取指定的交易
    pub fn total_vout_by_sequence(&self, n: u32) -> f64 {
        self.vout.iter().filter(|v| v.n == n).map(|v| v.value).sum()
    }
}

#[derive(Deserialize, Debug)]
pub struct Vin {
    pub txid: String,
    pub vout: u32,
    #[serde(rename = "scriptSig")]
    pub script_sig: ScriptSig,
    pub sequence: u32,
    pub txinwitness: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct ScriptSig {
    pub asm: String,
    pub hex: String,
}
#[derive(Deserialize, Debug)]
pub struct Vout {
    pub value: f64,
    pub n: u32,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: ScriptPubKey,
}

#[derive(Deserialize, Debug)]
pub struct ScriptPubKey {
    pub asm: String,
    pub desc: String,
    pub hex: String,
    pub address: Option<String>,
    #[serde(rename = "type")]
    pub types: String,
}

#[derive(Serialize, Debug)]
pub struct Inputs {
    pub txid: String,
    pub vout: u32,
}
impl From<&Utxo> for Inputs {
    fn from(utxo: &Utxo) -> Self {
        Self {
            txid: utxo.txid.clone(),
            vout: utxo.vout,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiBlock {
    pub page: u32,
    pub total_pages: u32,
    pub items_on_page: u32,
    pub hash: String,
    pub previous_block_hash: String,
    pub next_block_hash: Option<String>,
    pub height: u32,
    pub confirmations: u32,
    pub size: u32,
    pub time: u64,
    pub version: Option<u32>,
    pub merkle_root: String,
    pub nonce: String,
    pub bits: String,
    pub difficulty: String,
    pub tx_count: u32,
    pub txs: Vec<ApiTransaction>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcVin {
    pub coinbase: Option<String>,
    pub ismweb: bool,
    pub sequence: i64,
    pub txinwitness: Option<Vec<String>>,
    pub txid: Option<String>,
    pub vout: Option<u64>,
    pub script_sig: Option<ScriptSig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcVout {
    pub ismweb: bool,
    pub n: i32,
    pub script_pub_key: JsonRpcScriptPubKey,
    pub value: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcScriptPubKey {
    pub desc: Option<String>,
    pub asm: String,
    pub hex: String,
    pub r#type: String,
    pub req_sigs: Option<i32>,
    pub addresses: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcTx {
    pub blockhash: Option<String>,
    pub blocktime: Option<u64>,
    pub confirmations: Option<u64>,
    pub hash: String,
    pub hex: String,
    pub locktime: u64,
    pub size: u64,
    pub time: Option<u64>,
    pub txid: String,
    pub version: i32,
    pub vin: Vec<JsonRpcVin>,
    pub vout: Vec<JsonRpcVout>,
    pub vsize: u64,
    pub weight: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcBlock {
    pub bits: String,
    pub chainwork: String,
    pub confirmations: u64,
    pub difficulty: f64,
    pub hash: String,
    pub height: u64,
    pub mediantime: u64,
    pub merkleroot: String,
    pub mweb: Option<Mweb>,
    pub n_tx: u64,
    pub nextblockhash: String,
    pub nonce: u64,
    pub previousblockhash: String,
    pub size: u64,
    pub strippedsize: u64,
    pub time: u64,
    pub tx: Vec<JsonRpcTx>,
    pub version: u64,
    pub version_hex: String,
    pub weight: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionUtxo {
    pub txid: String,
    pub vout: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ValidateAddress {
    pub isvalid: bool,
    pub address: String,
    pub isscript: Option<bool>,
    pub iswitness: Option<bool>,
    pub ismweb: Option<bool>,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: Option<String>,
    pub witness_program: Option<String>,
    pub witness_version: Option<u64>,
    pub error: Option<String>,
    pub error_locations: Option<Vec<u64>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mweb {
    pub hash: Option<String>,
    pub height: Option<u64>,
    pub inputs: Vec<serde_json::Value>,
    pub kernel_offset: Option<String>,
    pub kernel_root: Option<String>,
    pub kernels: Vec<serde_json::Value>,
    pub leaf_root: Option<String>,
    pub num_kernels: Option<u64>,
    pub num_txos: Option<u64>,
    pub output_root: Option<String>,
    pub outputs: Vec<serde_json::Value>,
    pub stealth_offset: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LtcJsonRpcReq {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LtcJsonRpcRes {
    pub result: serde_json::Value,
    pub error: Option<ApiError>,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiTransaction {
    pub txid: String,
    pub version: Option<u32>,
    pub vin: Vec<ApiVin>,
    pub vout: Vec<ApiVout>,
    pub block_hash: Option<String>,
    pub block_height: i128,
    pub confirmations: u64,
    pub block_time: u64,
    pub size: Option<u64>,
    pub vsize: Option<u64>,
    pub value: String,
    pub value_in: String,
    pub fees: String,
    pub hex: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub value: u64,
}

impl ApiTransaction {
    pub fn get_from(&self) -> Vec<AddressInfo> {
        self.vin
            .iter()
            .filter(|v| v.is_address)
            .flat_map(|v| {
                let addresses = v.addresses.clone().unwrap_or_default();
                let value = v.value.parse::<u64>().unwrap_or_default(); // 假设v.value 是一个包含金额的字段
                addresses
                    .into_iter()
                    .map(move |address| AddressInfo { address, value })
            })
            .collect()
    }
    pub fn get_to(&self) -> Vec<AddressInfo> {
        self.vout
            .iter()
            .filter(|v| v.is_address)
            .flat_map(|v| {
                let addresses = v.addresses.clone();
                let value = v.value.parse::<u64>().unwrap_or_default(); // 假设v.value 是一个包含金额的字段
                addresses
                    .into_iter()
                    .map(move |address| AddressInfo { address, value })
            })
            .collect()
    }

    pub fn get_fees(&self) -> crate::Result<f64> {
        let res = unit::u256_from_str(&self.fees)?;
        Ok(unit::format_to_f64(res, LTC_DECIMAL)?)
    }

    pub fn get_value(&self) -> crate::Result<f64> {
        let res = unit::u256_from_str(&self.value)?;
        Ok(unit::format_to_f64(res, LTC_DECIMAL)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiVin {
    pub sequence: Option<u64>,
    pub n: Option<u64>,
    pub addresses: Option<Vec<String>>,
    pub value: String,
    pub is_address: bool,
    pub coinbase: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiVout {
    pub value: String,
    pub n: u64,
    pub addresses: Vec<String>,
    pub is_address: bool,
}

impl ValidateAddress {
    pub fn address_type(&self) -> Option<LtcAddressType> {
        if self.isscript.is_some() && !self.isscript.unwrap() {
            return Some(LtcAddressType::P2pkh);
        } else if self.isscript.is_some() && self.isscript.unwrap() {
            return Some(LtcAddressType::P2shWpkh);
        } else if self.iswitness.is_some()
            && self.iswitness.unwrap()
            && self.witness_version.is_some()
            && self.witness_version.unwrap() == 0
        {
            return Some(LtcAddressType::P2wpkh);
        } else if self.iswitness.is_some()
            && self.witness_version.is_some()
            && self.iswitness.unwrap()
            && self.witness_version.unwrap() == 1
        {
            return Some(LtcAddressType::P2tr);
        } else {
            return None;
        }
    }
}
