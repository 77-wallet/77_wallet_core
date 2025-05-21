use crate::ton::protocol::block::BlockTransactionExt;
use alloy::primitives::U256;
use wallet_transport::client::HttpClient;
use wallet_utils::unit;

use super::{
    params::{EstimateFeeParams, LocateTxParams, QueryTransParams},
    protocol::{
        account::{AccountTransactions, AddressInformation},
        block::{BlocksShards, ConsensusBlock, MasterChainInfo},
        common::{ConfigParams, RunGetMethodParams, RunGetMethodResp},
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

    // async fn do_get<T, R>(&self, endpoint: &str, params: T) -> crate::Result<R>
    // where
    //     T: serde::Serialize + std::fmt::Debug,
    //     R: serde::de::DeserializeOwned,
    // {
    //     let res = self
    //         .client
    //         .get_with_params::<T, TonResponse<R>>(endpoint, params)
    //         .await?;

    //     if !res.ok {
    //         Err(TransportError::NodeResponseError(NodeResponseError::new(
    //             res.code.unwrap_or_default(),
    //             res.error,
    //         )))?
    //     } else {
    //         Ok(res.result)
    //     }
    // }

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
        let payload = std::collections::HashMap::from([
            ("workchain", workchain.to_string()),
            ("shard", shard.to_string()),
            ("seqno", seqno.to_string()),
        ]);

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
