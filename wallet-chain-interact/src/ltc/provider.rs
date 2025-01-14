use super::{
    protocol::{
        other::FeeRate,
        transaction::{
            ApiBlock, ApiTransaction, ApiUtxo, EstimateFee, JsonRpcBlock, JsonRpcTx, LtcJsonRpcReq,
<<<<<<< HEAD
            LtcJsonRpcRes, TransactionUtxo, ValidateAddress,
=======
            LtcJsonRpcRes, ToAddressValue, TransactionUtxo,
>>>>>>> 6e0052e (ltc add findfree)
        },
        BlockHeader, OutInfo, ScanOut,
    },
    utxos::{Utxo, UtxoList},
};
use wallet_utils::Error as WalletError;
use wallet_utils::SerdeError;

use serde_json::{from_value, json};
use std::collections::HashMap;
use wallet_transport::{
    client::{HttpClient, RpcClient},
    types::JsonRpcParams,
};

pub struct ProviderConfig {
    pub rpc_url: String,
    pub rpc_auth: Option<RpcAuth>,
    pub access_key: Option<String>,
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
    api_client: HttpClient,
}

pub const API_ENPOINT: &'static str = "";

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

        let mut header_map_http = header_opt.clone().unwrap_or_else(HashMap::new);
        if let Some(access_key) = config.access_key {
            header_map_http.insert("access-key".to_owned(), access_key);
        }
        let mut header_map_api = header_opt.unwrap_or_else(HashMap::new);
        if let Some(api_key) = config.http_api_key {
            header_map_api.insert("api-key".to_owned(), api_key);
        }

        let header_map_http = (!header_map_http.is_empty()).then_some(header_map_http);
        let http_client = HttpClient::new(&config.rpc_url, header_map_http, timeout)?;

        let header_map_api = (!header_map_api.is_empty()).then_some(header_map_api);
        let api_client = HttpClient::new(&config.http_url, header_map_api, timeout)?;

        Ok(Self {
            client,
            http_client,
            api_client,
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

                let result = self.client.invoke_request::<_, ScanOut>(params).await?;

                let mut utxo = result
                    .unspents
                    .iter()
                    .map(Utxo::from)
                    .collect::<Vec<Utxo>>();
                utxo.sort_by(|a, b| a.value.cmp(&b.value));
                Ok(UtxoList(utxo))
            }
            _ => {
                let url = format!("{}/utxo/{}", API_ENPOINT, address);

                let mut params = HashMap::new();
                params.insert("confirmed", "false");

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

                let reuslt = self.client.invoke_request::<_, FeeRate>(params).await?;
                Ok(reuslt)
            }
        }
    }

    pub async fn send_raw_transaction(&self, hex_raw: &str) -> crate::Result<String> {
        let params = JsonRpcParams::default()
            .method("sendrawtransaction")
            .params(vec![hex_raw]);

        Ok(self.client.invoke_request::<_, String>(params).await?)
    }

    pub async fn utxo_out(&self, tx_id: &str, index: u32) -> crate::Result<OutInfo> {
        let txid = serde_json::Value::from(tx_id);
        let index = serde_json::Value::from(index);

        let params = JsonRpcParams::default()
            .method("gettxout")
            .params(vec![txid, index]);

        Ok(self.client.invoke_request::<_, OutInfo>(params).await?)
    }

    pub async fn block_header(&self, block_hash: &str) -> crate::Result<BlockHeader> {
        let params = JsonRpcParams::default()
            .method("getblockheader")
            .params(vec![block_hash]);

        Ok(self.client.invoke_request::<_, BlockHeader>(params).await?)
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

        Ok(self.client.invoke_request::<_, T>(params).await?)
    }

    pub async fn block_heigh(&self) -> crate::Result<u64> {
        let params = JsonRpcParams::<Vec<String>>::default().method("getblockcount");

        Ok(self.client.invoke_request::<_, u64>(params).await?)
    }

    pub async fn get_transaction_from_api(&self, hash: &str) -> crate::Result<ApiTransaction> {
        let url = format!("{}/tx/{}", API_ENPOINT, hash);
        let res = self.http_client.get_request::<ApiTransaction>(&url).await?;
        Ok(res)
    }

    pub async fn get_block_from_api(&self, height: u64) -> crate::Result<ApiBlock> {
        let url = format!("block/{}", height);
        let res = self.api_client.get_request::<ApiBlock>(&url).await?;
        Ok(res)
    }
    pub async fn get_uxto_from_api(&self, addr: &str) -> crate::Result<Vec<ApiUtxo>> {
        let url = format!("utxo/{}", addr);
        let res = self.api_client.get_request::<Vec<ApiUtxo>>(&url).await?;
        Ok(res)
    }

    pub async fn get_transaction_from_json_rpc(&self, hash: &str) -> crate::Result<JsonRpcTx> {
        let payload = LtcJsonRpcReq {
            jsonrpc: "2.0".to_string(),
            id: "1".to_string(),
            method: "getrawtransaction".to_string(),
            params: vec![json!(hash), json!(true)],
        };
        let res = self
            .http_client
            .post_request::<LtcJsonRpcReq, LtcJsonRpcRes>("", payload)
            .await?;

        let res = from_value(res.result.clone()).map_err(|e| {
            WalletError::Serde(SerdeError::Deserialize(format!(
                "error = {} value = {}",
                e, res.result
            )))
        })?;

        Ok(res)
    }

    pub async fn get_block_hash_from_json_rpc(&self, height: u32) -> crate::Result<String> {
        let payload = LtcJsonRpcReq {
            jsonrpc: "2.0".to_string(),
            id: "1".to_string(),
            method: "getblockhash".to_string(),
            params: vec![json!(height)],
        };
        let res = self
            .http_client
            .post_request::<LtcJsonRpcReq, LtcJsonRpcRes>("", payload)
            .await?;

        let res = from_value(res.result.clone()).map_err(|e| {
            WalletError::Serde(SerdeError::Deserialize(format!(
                "error = {} value = {}",
                e, res.result
            )))
        })?;

        Ok(res)
    }

    pub async fn get_block_from_json_rpc(&self, hash: &str) -> crate::Result<JsonRpcBlock> {
        let payload = LtcJsonRpcReq {
            jsonrpc: "2.0".to_string(),
            id: "1".to_string(),
            method: "getblock".to_string(),
            params: vec![json!(hash), json!(2)],
        };

        let res = self
            .http_client
            .post_request::<LtcJsonRpcReq, LtcJsonRpcRes>("", payload)
            .await?;

        tracing::debug!("block result: {:?}", res.result.to_string());

        let res = from_value(res.result.clone()).map_err(|e| {
            WalletError::Serde(SerdeError::Deserialize(format!(
                "error = {} value = {}",
                e, res.result
            )))
        })?;
        Ok(res)
    }

    pub async fn estimate_fee_from_json_rpc(&self, blocks: u64) -> crate::Result<EstimateFee> {
        let payload = LtcJsonRpcReq {
            jsonrpc: "2.0".to_string(),
            id: "1".to_string(),
            method: "estimatesmartfee".to_string(),
            params: vec![json!(blocks)],
        };
        let res = self
            .http_client
            .post_request::<LtcJsonRpcReq, LtcJsonRpcRes>("", payload)
            .await?;

        let res = from_value(res.result.clone()).map_err(|e| {
            WalletError::Serde(SerdeError::Deserialize(format!(
                "error = {} value = {}",
                e, res.result
            )))
        })?;

        Ok(res)
    }

    pub async fn validate_address_from_json_rpc(
        &self,
        addr: &str,
    ) -> crate::Result<ValidateAddress> {
        let payload = LtcJsonRpcReq {
            jsonrpc: "2.0".to_string(),
            id: "1".to_string(),
            method: "validateaddress".to_string(),
            params: vec![json!(addr)],
        };
        let res = self
            .http_client
            .post_request::<LtcJsonRpcReq, LtcJsonRpcRes>("", payload)
            .await?;

        let res = from_value(res.result.clone()).map_err(|e| {
            WalletError::Serde(SerdeError::Deserialize(format!(
                "error = {} value = {}",
                e, res.result
            )))
        })?;

        Ok(res)
    }

    pub async fn create_raw_transaction_from_json_rpc(
        &self,
        txs: &Vec<TransactionUtxo>,
        addr: &str,
        value: f64,
    ) -> crate::Result<String> {
        let addr_value = HashMap::from([(addr, value)]);

        let payload = LtcJsonRpcReq {
            jsonrpc: "2.0".to_string(),
            id: "1".to_string(),
            method: "createrawtransaction".to_string(),
            params: vec![json!(txs), json!(addr_value)],
        };
        let res = self
            .http_client
            .post_request::<LtcJsonRpcReq, LtcJsonRpcRes>("", payload)
            .await?;

        let res = from_value(res.result.clone()).map_err(|e| {
            WalletError::Serde(SerdeError::Deserialize(format!(
                "error = {} value = {}",
                e, res.result
            )))
        })?;

        Ok(res)
    }
}
