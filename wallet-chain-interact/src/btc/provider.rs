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
        timeout: Option<std::time::Duration>,
    ) -> crate::Result<Self> {
        let client = if let Some(auth) = config.rpc_auth {
            RpcClient::new_with_base_auth(&config.rpc_url, &auth.user, &auth.password, timeout)?
        } else {
            RpcClient::new(&config.rpc_url, header_opt.clone(), timeout)?
        };

        let mut header_map = header_opt.unwrap_or_else(HashMap::new);
        if let Some(api_key) = config.http_api_key {
            header_map.insert("api-key".to_owned(), api_key);
        }

        let header_map = (!header_map.is_empty()).then_some(header_map);
        let http_client = HttpClient::new(&config.http_url, header_map, timeout)?;

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
                let url = format!("utxo/{}", address);

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
        let url = format!("tx/{}", hash);
        let res = self.http_client.get_request::<ApiTransaction>(&url).await?;
        Ok(res)
    }
    pub async fn get_block_from_api(&self, hash: &str, page: u32) -> crate::Result<ApiBlock> {
        let url = format!("block/{}?page={}", hash, page);
        let res = self.http_client.get_request::<ApiBlock>(&url).await?;
        Ok(res)
    }
}
