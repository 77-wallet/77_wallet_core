use super::{
    consts::{EXPEND_FEE_RATE, MAX_FEE_RATE},
    protocol::{
        BlockHeader, OutInfo,
        other::FeeRate,
        transaction::{ApiBlock, ApiTransaction, JsonRpcTx, TransactionUtxo, ValidateAddress},
    },
    utxos::{Utxo, UtxoList},
};

use serde_json::json;
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
}

pub const API_ENPOINT: &'static str = "book/api/v2";

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

        let mut header_map_api = header_opt.unwrap_or_else(HashMap::new);
        if let Some(api_key) = config.http_api_key {
            header_map_api.insert("api-key".to_owned(), api_key);
        }

        let header_map_api = (!header_map_api.is_empty()).then_some(header_map_api);
        let http_client = HttpClient::new(&config.http_url, header_map_api, timeout)?;

        Ok(Self {
            client,
            http_client,
        })
    }

    pub async fn utxos(
        &self,
        address: &str,
        _network: wallet_types::chain::network::NetworkKind,
    ) -> crate::Result<UtxoList> {
        let mut utxo = self.get_uxto_from_api(address).await?;

        utxo.sort_by(|a, b| a.value.cmp(&b.value));

        Ok(UtxoList(utxo))
    }

    pub async fn fetch_fee_rate(&self, blocks: u32) -> crate::Result<litecoin::Amount> {
        let res = self.estimate_fee(blocks as u64).await?;

        let fee_rate = litecoin::Amount::from_sat((res.fee_rate * 100_000.0).round() as u64);

        // 扩大推荐费用,加快打包
        let fee_rate = fee_rate * EXPEND_FEE_RATE;
        let max_fee_rate = litecoin::Amount::from_sat(MAX_FEE_RATE);
        if fee_rate > max_fee_rate {
            return Err(crate::UtxoError::ExceedsMaxFeeRate.into());
        }

        Ok(fee_rate)
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

    // 1
    pub async fn block_heigh(&self) -> crate::Result<u64> {
        let params = JsonRpcParams::<Vec<String>>::default().method("getblockcount");

        Ok(self.client.invoke_request::<_, u64>(params).await?)
    }

    // 2
    pub async fn get_transaction_from_api(&self, hash: &str) -> crate::Result<ApiTransaction> {
        let url = format!("{}/tx/{}", API_ENPOINT, hash);

        Ok(self.http_client.get_request::<ApiTransaction>(&url).await?)
    }

    // 2
    pub async fn get_block_from_api(&self, height: u64, page: u32) -> crate::Result<ApiBlock> {
        let url = format!("{}/block/{}?page={}", API_ENPOINT, height, page);
        let res = self.http_client.get_request::<ApiBlock>(&url).await?;
        Ok(res)
    }
    pub async fn get_uxto_from_api(&self, addr: &str) -> crate::Result<Vec<Utxo>> {
        let url = format!("{}/utxo/{}", API_ENPOINT, addr);

        Ok(self.http_client.get_request::<Vec<Utxo>>(&url).await?)
    }

    pub async fn get_transaction_from_json_rpc(&self, hash: &str) -> crate::Result<JsonRpcTx> {
        let params = JsonRpcParams::default()
            .params(vec![json!(hash), json!(true)])
            .method("getrawtransaction");

        Ok(self.client.invoke_request::<_, JsonRpcTx>(params).await?)
    }

    // 获取原始的费率
    pub async fn estimate_fee(&self, blocks: u64) -> crate::Result<FeeRate> {
        let params = JsonRpcParams::default()
            .method("estimatesmartfee")
            .params(vec![blocks]);

        Ok(self.client.invoke_request::<_, FeeRate>(params).await?)
    }

    pub async fn validate_address_from_json_rpc(
        &self,
        addr: &str,
    ) -> crate::Result<ValidateAddress> {
        let params = JsonRpcParams::default()
            .method("validateaddress")
            .params(vec![addr]);

        Ok(self
            .client
            .invoke_request::<_, ValidateAddress>(params)
            .await?)
    }

    pub async fn create_raw_transaction_from_json_rpc(
        &self,
        txs: &Vec<TransactionUtxo>,
        addr: &str,
        value: f64,
    ) -> crate::Result<String> {
        let addr_value = HashMap::from([(addr, value)]);

        let params = JsonRpcParams::default()
            .method("createrawtransaction")
            .params(vec![json!(txs), json!(addr_value)]);

        Ok(self.client.invoke_request::<_, String>(params).await?)
    }
}
