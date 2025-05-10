use crate::ton::protocol::block::BlockTransactionExt;
use alloy::primitives::U256;
use serde_json::json;
use wallet_transport::client::HttpClient;
use wallet_utils::unit;

use super::{
    params::{EstimateFeeParams, LocateTxParams},
    protocol::{
        account::AddressInformation,
        block::{BlocksShards, ConsensusBlock, MasterChainInfo},
        common::{ConfigParams, RunGetMethodParams, RunGetMethodResp},
        jettons::TokenDataResp,
        transaction::{AddressId, EstimateFeeResp, RawTransaction, SendBocReturn},
    },
};

#[derive(Debug, serde::Deserialize)]
pub struct TonResponse<T> {
    pub ok: bool,
    pub result: T,
}

pub struct Provider {
    pub client: HttpClient,
}

impl Provider {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    pub async fn balance(&self, addr: &str) -> crate::Result<U256> {
        let params = std::collections::HashMap::from([("address", addr)]);

        let res = self
            .client
            .get_with_params::<_, TonResponse<String>>("getAddressBalance", params)
            .await?;

        Ok(unit::u256_from_str(&res.result)?)
    }

    pub async fn account_info(&self, addr: &str) -> crate::Result<AddressInformation> {
        let params = std::collections::HashMap::from([("address", addr)]);

        let res = self
            .client
            .get_with_params::<_, TonResponse<AddressInformation>>("getAddressInformation", params)
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

    pub async fn get_transaction(&self, address: &str) -> crate::Result<Vec<RawTransaction>> {
        let params = std::collections::HashMap::from([("address", address)]);

        let res = self
            .client
            .get_with_params::<_, TonResponse<Vec<RawTransaction>>>("getTransactions", params)
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

    pub async fn token_data(&self, addr: &str) -> crate::Result<TokenDataResp> {
        let payload = std::collections::HashMap::from([("address", addr)]);

        let res = self
            .client
            .get_with_params::<_, TonResponse<TokenDataResp>>("getTokenData", payload)
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
        let res = self
            .client
            .post_request::<_, TonResponse<RunGetMethodResp>>("runGetMethod", params)
            .await?;

        Ok(res.result)
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
    ) -> Result<BlockTransactionExt<AddressId>, wallet_transport::TransportError> {
        let payload = json!({
            "workchain":workchain,
            "shard":shard,
            "seqno":seqno,
        });

        let res = self
            .client
            .get_with_params::<_, TonResponse<BlockTransactionExt<AddressId>>>(
                "getBlockTransactionsExt",
                payload,
            )
            .await?;

        Ok(res.result)
    }
}
