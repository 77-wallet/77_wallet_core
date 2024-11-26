use super::{
    protocol::{
        other::FeeRate,
        transaction::{ApiBlock, ApiTransaction},
        BlockHeader, OutInfo, ScanOut,
    },
    utxos::{Utxo, UtxoList},
};
use serde_json::json;
use std::collections::HashMap;
use wallet_transport::{
    client::{HttpClient, RpcClient},
    types::{JsonRpcParams, JsonRpcResult},
};

pub struct ProviderConfig {
    pub rpc_url: String,
    pub rpc_auth: Option<RpcAuth>,
    pub http_url: String,
    pub http_api_key: Option<String>,
}
pub struct RpcAuth {
    pub user: String,
    pub password: String,
}

pub struct Provider {
    client: RpcClient,
    http_client: HttpClient,
}
impl Provider {
    pub fn new(
        config: ProviderConfig,
        header_opt: Option<HashMap<String, String>>,
    ) -> crate::Result<Self> {
        let client = if let Some(auth) = config.rpc_auth {
            RpcClient::new_with_base_auth(&config.rpc_url, &auth.user, &auth.password)?
        } else {
            RpcClient::new(&config.rpc_url, header_opt)?
        };

        let header_map = if let Some(api_key) = config.http_api_key {
            let mut header_map = HashMap::new();
            header_map.insert("api-key".to_owned(), api_key);
            Some(header_map)
        } else {
            None
        };

        let http_client = HttpClient::new(&config.http_url, header_map)?;

        Ok(Self {
            client,
            http_client,
        })
    }

    pub async fn utxos(
        &self,
        address: &str,
        network: wallet_types::chain::network::NetworkKind,
    ) -> crate::Result<UtxoList> {
        match network {
            wallet_types::chain::network::NetworkKind::Regtest => {
                let json_str = format!(r#"["start", [{{"desc":"addr({})"}}]]"#, address);
                let v: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
                let params = JsonRpcParams::default().method("scantxoutset").params(v);

                let result = self
                    .client
                    .set_params(params)
                    .send::<JsonRpcResult<ScanOut>>()
                    .await?;

                let mut utxo = result
                    .result
                    .unspents
                    .iter()
                    .map(Utxo::from)
                    .collect::<Vec<Utxo>>();
                utxo.sort_by(|a, b| a.value.cmp(&b.value));
                Ok(UtxoList(utxo))
            }
            _ => {
                let url = format!("v2/utxo/{}", address);

                let mut params = HashMap::new();
                params.insert("confirmed", "true");

                let mut utxo = self
                    .http_client
                    .get(&url)
                    .query(params)
                    .send::<Vec<Utxo>>()
                    .await?;
                utxo.sort_by(|a, b| a.value.cmp(&b.value));
                Ok(UtxoList(utxo))
            }
        }
    }

    pub async fn fetch_fee_rate(
        &self,
        blocks: u32,
        network: wallet_types::chain::network::NetworkKind,
    ) -> crate::Result<bitcoin::Amount> {
        let res = self.estimate_smart_fee(blocks, network).await?;
        Ok(bitcoin::Amount::from_sat(
            (res.fee_rate * 100_000.0).round() as u64,
        ))
    }

    pub async fn estimate_smart_fee(
        &self,
        blocks: u32,
        network: wallet_types::chain::network::NetworkKind,
    ) -> crate::Result<FeeRate> {
        match network {
            wallet_types::chain::network::NetworkKind::Regtest => {
                // 本地回归测试网络写死
                Ok(FeeRate {
                    fee_rate: 0.000048779,
                    blocks: 2,
                })
            }
            _ => {
                let params = JsonRpcParams::default()
                    .method("estimatesmartfee")
                    .params(vec![blocks]);

                let result = self
                    .client
                    .set_params(params)
                    .send::<JsonRpcResult<FeeRate>>()
                    .await?;

                Ok(result.result)
            }
        }
    }

    pub async fn send_raw_transaction(&self, hex_raw: &str) -> crate::Result<String> {
        let params = JsonRpcParams::default()
            .method("sendrawtransaction")
            .params(vec![hex_raw]);

        let result = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<String>>()
            .await?;
        Ok(result.result)
    }

    pub async fn utxo_out(&self, tx_id: &str, index: u32) -> crate::Result<OutInfo> {
        let txid = serde_json::Value::from(tx_id);
        let index = serde_json::Value::from(index);

        let params = JsonRpcParams::default()
            .method("gettxout")
            .params(vec![txid, index]);

        let result = self
            .client
            .set_params(params)
            .send_json_rpc::<OutInfo>()
            .await?;
        Ok(result)
    }

    pub async fn block_header(&self, block_hash: &str) -> crate::Result<BlockHeader> {
        let params = JsonRpcParams::default()
            .method("getblockheader")
            .params(vec![block_hash]);

        let result = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<BlockHeader>>()
            .await?;

        Ok(result.result)
    }

    pub async fn query_transaction<T>(&self, txid: &str, verbose: bool) -> crate::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let tx_id = serde_json::Value::from(txid);
        let verbose = serde_json::Value::from(verbose);

        let params = JsonRpcParams::default()
            .method("getrawtransaction")
            .params(vec![tx_id, verbose]);

        let result = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<T>>()
            .await?;

        Ok(result.result)
    }

    pub async fn block_heigh(&self) -> crate::Result<u64> {
        let params = JsonRpcParams::<Vec<String>>::default().method("getblockcount");

        let result = self
            .client
            .set_params(params)
            .send_json_rpc::<u64>()
            .await?;

        Ok(result)
    }

    pub async fn block_info(&self, block_hash: &str) -> crate::Result<String> {
        let tx_id = json!(block_hash);
        let options = json!(0);

        let params = JsonRpcParams::default()
            .method("getblock")
            .params(vec![tx_id, options]);

        let res = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;
        Ok(res)
    }

    pub async fn get_transaction_from_api(&self, hash: &str) -> crate::Result<ApiTransaction> {
        let url = format!("v2/tx/{}", hash);
        let res = self.http_client.get_request::<ApiTransaction>(&url).await?;
        Ok(res)
    }
    pub async fn get_block_from_api(&self, hash: &str, page: u32) -> crate::Result<ApiBlock> {
        let url = format!("v2/block/{}?page={}", hash, page);
        let res = self.http_client.get_request::<ApiBlock>(&url).await?;
        Ok(res)
    }
}

// pub fn parse_transaction(
//     &self,
//     tx: bitcoin::Transaction,
//     network: bitcoin::network::Network,
// ) -> crate::Result<()> {
//     tracing::info!("{:?}", tx);
//     for tx_in in tx.input.iter() {
//         self.detect_input_address(tx_in, network);
//     }
//     Ok(())
// }

// pub fn detect_input_address(
//     &self,
//     tx_in: &bitcoin::TxIn,
//     network: bitcoin::network::Network,
// ) -> Option<bitcoin::Address> {
//     // sig 不为空  以及 witness 为空时 可以判断是  传统地址
//     if !tx_in.script_sig.is_empty() && tx_in.witness.is_empty() {
//         let elements: Vec<_> = tx_in.script_sig.instructions().collect();
//         // 只有两个元素代表 签名数据和 公钥
//         if elements.len() == 2 {
//             if let Some(Ok(Instruction::PushBytes(pubkey_bytes))) = elements.get(1) {
//                 let pubkey_slice = pubkey_bytes.as_bytes();
//                 if let Ok(pubkey) = bitcoin::PublicKey::from_slice(pubkey_slice) {
//                     let address = bitcoin::Address::p2pkh(&pubkey, network);
//                     tracing::warn!("address {}", address);
//                     return Some(bitcoin::Address::p2pkh(&pubkey, network));
//                 }
//             }
//         } else {
//             // 获取最后一个元素,redeemScript
//             if let Some(Ok(Instruction::PushBytes(redeem_script_bytes))) = elements.last() {
//                 let pubkey_slice = redeem_script_bytes.as_bytes().to_vec();
//                 let redeem_script = bitcoin::ScriptBuf::from_bytes(pubkey_slice);
//                 let script = redeem_script.as_script();

//                 return bitcoin::Address::p2sh(script, network).ok();
//             }
//         }
//     } else if !tx_in.witness.is_empty() {
//         let witness = &tx_in.witness;
//         if witness.len() == 2 {
//             // 几乎可以确定是p2wpkh
//             let pubkey_slice = witness.last().unwrap();
//             if let Ok(pubkey) = bitcoin::CompressedPublicKey::from_slice(pubkey_slice) {
//                 let address = bitcoin::Address::p2wpkh(&pubkey, network);
//                 return Some(address);
//             }
//         } else if witness.len() == 1 {
//             // 几乎判断是p2tr
//             let pubkey_slice = witness.last().unwrap();
//             if let Ok(internal_key) = bitcoin::XOnlyPublicKey::from_slice(pubkey_slice) {
//                 let secp = bitcoin::secp256k1::Secp256k1::new();
//                 let address = bitcoin::Address::p2tr(&secp, internal_key, None, network);
//                 return Some(address);
//             }
//         } else {
//             let script_slice = witness.last().unwrap();
//             let witness_script = bitcoin::ScriptBuf::from_bytes(script_slice.to_vec());

//             return bitcoin::Address::from_script(&witness_script, network).ok();
//         }
//     }
//     None
// }
