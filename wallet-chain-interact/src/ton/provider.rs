use crate::ton::protocol::block::BlockTransactionExt;
use alloy::primitives::U256;
use wallet_transport::{client::HttpClient, types::JsonRpcParams};
use wallet_utils::unit;

use super::{
    params::{EstimateFeeParams, LocateTxParams, QueryTransParams},
    protocol::{
        account::{AccountTransactions, AddressInformation},
        block::{BlocksShards, ConsensusBlock, MasterChainInfo},
        common::{ConfigParams, RunGetMethodParams, RunGetMethodResp},
        jettons::JettonMeta,
        transaction::{AddressId, EstimateFeeResp, RawTransaction, SendBocReturn},
    },
};

#[derive(Debug, serde::Deserialize)]
pub struct TonResponse<T> {
    pub ok: bool,
    pub result: T,
    pub error: Option<String>,
    pub code: Option<i64>,
}

pub struct Provider {
    pub client: HttpClient,
}

impl Provider {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    pub async fn json_rpc<T, R>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, wallet_transport::TransportError>
    where
        T: serde::Serialize + std::fmt::Debug,
        R: serde::de::DeserializeOwned,
    {
        let params = JsonRpcParams::default().method(method).params(params);

        let result = self
            .client
            .invoke_request::<_, TonResponse<R>>(Some("jsonRPC"), params)
            .await?;

        if !result.ok {
            return Err(wallet_transport::TransportError::NodeResponseError(
                wallet_transport::errors::NodeResponseError::new(
                    result.code.unwrap_or(0),
                    result.error,
                ),
            ));
        }

        Ok(result.result)
    }

    pub async fn balance(&self, addr: &str) -> crate::Result<U256> {
        let params = std::collections::HashMap::from([("address", addr)]);

        let res = self
            .client
            .get_with_params::<_, TonResponse<String>>("getAddressBalance", params)
            .await?;

        Ok(unit::u256_from_str(&res.result)?)
    }

    pub async fn address_information(&self, addr: &str) -> crate::Result<AddressInformation> {
        let params = std::collections::HashMap::from([("address", addr)]);

        let res = self
            .client
            .get_with_params::<_, TonResponse<AddressInformation>>("getAddressInformation", params)
            .await?;

        Ok(res.result)
    }

    pub async fn address_ext_information(&self, addr: &str) -> crate::Result<AddressInformation> {
        let params = std::collections::HashMap::from([("address", addr)]);

        let res = self
            .client
            .get_with_params::<_, TonResponse<AddressInformation>>(
                "getExtendedAddressInformation",
                params,
            )
            .await?;

        Ok(res.result)
    }

    pub async fn master_chain_info(&self) -> crate::Result<MasterChainInfo> {
        let res = self
            .client
            .get_request::<TonResponse<MasterChainInfo>>("getMasterchainInfo")
            .await?;
        Ok(res.result)
    }

    pub async fn block_shards(&self, seqno: i64) -> crate::Result<BlocksShards> {
        let params = std::collections::HashMap::from([("seqno", seqno)]);

        let res = self
            .client
            .get_with_params::<_, TonResponse<BlocksShards>>("shards", params)
            .await?;

        Ok(res.result)
    }

    pub async fn get_transaction(
        &self,
        payload: &QueryTransParams,
    ) -> crate::Result<AccountTransactions> {
        let res = self
            .client
            .get_with_params::<_, TonResponse<AccountTransactions>>("getTransactions", payload)
            .await?;

        Ok(res.result)
    }

    // 向后定位交易
    pub async fn try_locate_result_tx(
        &self,
        locate: LocateTxParams,
    ) -> crate::Result<RawTransaction> {
        let res = self
            .client
            .get_with_params::<_, TonResponse<RawTransaction>>("tryLocateResultTx", locate)
            .await?;

        Ok(res.result)
    }

    // 向前定位交易
    pub async fn try_locate_source_tx(
        &self,
        locate: LocateTxParams,
    ) -> crate::Result<RawTransaction> {
        let res = self
            .client
            .get_with_params::<_, TonResponse<RawTransaction>>("tryLocateSourceTx", locate)
            .await?;

        Ok(res.result)
    }

    // 发送原始的交易数据
    pub async fn send_boc_return(&self, body: String) -> crate::Result<String> {
        let data = std::collections::HashMap::from([("boc", body)]);

        let res = self
            .client
            .post_request::<_, TonResponse<SendBocReturn>>("sendBocReturnHash", data)
            .await?;

        Ok(res.result.hash)
    }

    pub async fn estimate_fee(&self, params: EstimateFeeParams) -> crate::Result<EstimateFeeResp> {
        let res = self
            .client
            .post_request::<_, TonResponse<EstimateFeeResp>>("estimateFee", params)
            .await?;

        Ok(res.result)
    }

    // jetton wallet ,jetton master,nft
    pub async fn token_data<T>(&self, addr: &str) -> crate::Result<T>
    where
        T: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let payload = std::collections::HashMap::from([("address", addr)]);

        let res = self
            .client
            .get_with_params::<_, TonResponse<T>>("getTokenData", payload)
            .await?;

        Ok(res.result)
    }

    // run get method
    pub async fn run_get_method<T>(
        &self,
        params: RunGetMethodParams<T>,
    ) -> Result<RunGetMethodResp, wallet_transport::TransportError>
    where
        T: serde::Serialize + std::fmt::Debug,
    {
        // let res = self
        //     .client
        //     .post_request::<_, TonResponse<RunGetMethodResp>>("runGetMethod", params)
        //     .await?;

        let res = self
            .json_rpc::<_, RunGetMethodResp>("runGetMethod", params)
            .await?;

        Ok(res)
    }

    pub async fn config_params(
        &self,
        config_id: u32,
    ) -> Result<ConfigParams, wallet_transport::TransportError> {
        let payload = std::collections::HashMap::from([("config_id", config_id)]);

        let res = self
            .client
            .get_with_params::<_, TonResponse<ConfigParams>>("getConfigParam", payload)
            .await?;

        Ok(res.result)
    }

    pub async fn consensus_block(
        &self,
    ) -> Result<ConsensusBlock, wallet_transport::TransportError> {
        let res = self
            .client
            .get_request::<TonResponse<ConsensusBlock>>("getConsensusBlock")
            .await?;

        Ok(res.result)
    }

    pub async fn get_block_transaction(
        &self,
        workchain: u64,
        shard: &str,
        seqno: u32,
    ) -> crate::Result<BlockTransactionExt<AddressId>> {
        let payload = std::collections::HashMap::from([
            ("workchain", workchain.to_string()),
            ("shard", shard.to_string()),
            ("seqno", seqno.to_string()),
            ("count", 200.to_string()),
        ]);

        let res = self
            .client
            .get_with_params::<_, TonResponse<BlockTransactionExt<AddressId>>>(
                "getBlockTransactionsExt",
                payload,
            )
            .await?;

        // let res = self
        //     .json_rpc::<_, TonResponse<BlockTransactionExt<AddressId>>>(
        //         "getBlockTransactionsExt",
        //         payload,
        //     )
        //     .await?;

        Ok(res.result)
    }

    pub async fn get_token_meta(&self, uri: &str) -> crate::Result<JettonMeta> {
        let mete = self
            .client
            .client
            .get(uri)
            .send()
            .await
            .map_err(|e| wallet_utils::Error::Http(e.into()))?;
        let content = mete
            .text()
            .await
            .map_err(|e| wallet_utils::Error::Http(e.into()))?;

        Ok(wallet_utils::serde_func::serde_from_str::<JettonMeta>(
            &content,
        )?)
    }
}
