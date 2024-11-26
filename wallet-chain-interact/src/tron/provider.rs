use super::{
    operations::{
        contract::{ConstantContract, TriggerContractParameter, TriggerContractResult},
        transfer::{ContractTransferResp, TronTransferResp},
        RawTransactionParams, TronTransactionResponse,
    },
    params::{self, ResourceConsumer, ResourceType},
    protocol::{
        account::{
            AccountResourceDetail, CanWithdrawUnfreezeAmount, DelegateResp, FreezeBalanceResp,
            TronAccount, UnDelegateResp, UnFreezeBalanceResp, WithdrawExpire,
        },
        block::TronBlock,
        chain_parameter::ChainParameter,
        receipt::TransactionInfo,
        transaction::{BaseTransaction, CreateTransactionResp, SendRawTransactionResp},
    },
};
use crate::tron::{params::Resource, protocol::transaction::SendRawTransactionParams};
use serde_json::json;
use std::{collections::HashMap, fmt::Debug};
use wallet_transport::client::HttpClient;
use wallet_utils::hex_func;

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum NodeResponse<R, E> {
    Success(R),
    Fail(E),
}

// 合约触发的错误
#[derive(Debug, serde::Deserialize)]
struct ContractError {
    pub result: ContractErrorResult,
}

#[derive(Debug, serde::Deserialize)]
struct ContractErrorResult {
    #[allow(unused)]
    pub code: String,
    pub message: String,
}

pub struct Provider {
    client: HttpClient,
}

impl Provider {
    pub fn new(http_client: HttpClient) -> crate::Result<Self> {
        Ok(Self {
            client: http_client,
        })
    }

    // 不去匹配错误与成功的响应(作为过渡使用)
    pub async fn do_request<T, R>(&self, endpoint: &str, params: Option<T>) -> crate::Result<R>
    where
        T: serde::Serialize + Debug,
        R: serde::de::DeserializeOwned,
    {
        let request = self.client.post(endpoint);

        let request = if let Some(params) = params {
            request.json(&params)
        } else {
            request
        };

        Ok(request.send::<R>().await?)
    }

    // 合约相关的调用、匹配成功和失败的情况
    async fn do_contract_request<T, R>(&self, endpoint: &str, params: Option<T>) -> crate::Result<R>
    where
        T: serde::Serialize + Debug,
        R: serde::de::DeserializeOwned,
    {
        let request = self.client.post(endpoint);
        let request = if let Some(params) = params {
            request.json(&params)
        } else {
            request
        };

        match request.send::<NodeResponse<R, ContractError>>().await? {
            NodeResponse::Success(r) => Ok(r),
            NodeResponse::Fail(err) => {
                let error_msg = hex_func::hex_to_utf8(&err.result.message)?;
                Err(crate::Error::RpcError(error_msg))
            }
        }
    }

    pub async fn get_block(&self) -> crate::Result<TronBlock> {
        self.do_request::<_, TronBlock>("wallet/getnowblock", None::<()>)
            .await
    }

    pub async fn create_transaction(
        &self,
        params: TronTransferResp,
    ) -> crate::Result<TronTransactionResponse<TronTransferResp>> {
        let res = self
            .do_request::<_, _>("wallet/createtransaction", Some(params))
            .await?;
        Ok(res)
    }

    // get account info
    pub async fn account_info(&self, account: &str) -> crate::Result<TronAccount> {
        let mut params = HashMap::from([("address", account)]);
        if account.starts_with("T") {
            params.insert("visible", "true");
        }

        self.do_request::<_, TronAccount>("wallet/getaccount", Some(params))
            .await
    }

    // get account resource
    pub async fn account_resource(&self, account: &str) -> crate::Result<AccountResourceDetail> {
        let mut params = HashMap::from([("address", account)]);
        if account.starts_with("T") {
            params.insert("visible", "true");
        }

        let res = self
            .do_request::<_, AccountResourceDetail>("wallet/getaccountresource", Some(params))
            .await?;
        Ok(res)
    }

    // only constant smart contract used to get contract information or estimate energy
    pub async fn trigger_constant_contract(
        &self,
        trigger: TriggerContractParameter,
    ) -> crate::Result<ConstantContract<ContractTransferResp>> {
        let result = self
            .do_contract_request::<_, _>("wallet/triggerconstantcontract", Some(trigger))
            .await?;
        Ok(result)
    }

    // build contract transaction
    pub async fn trigger_smart_contract(
        &self,
        trigger: TriggerContractParameter,
    ) -> crate::Result<TriggerContractResult<ContractTransferResp>> {
        let result = self
            .do_contract_request::<_, _>("wallet/triggersmartcontract", Some(trigger))
            .await?;
        Ok(result)
    }

    // 查询交易信息
    pub async fn query_tx_info(&self, tx_hash: &str) -> crate::Result<TransactionInfo> {
        let params = HashMap::from([("value", tx_hash)]);
        let result = self
            .do_request::<_, TransactionInfo>("wallet/gettransactioninfobyid", Some(params))
            .await?;
        Ok(result)
    }

    // exec raw transaction
    pub async fn exec_raw_transaction<T>(
        &self,
        raw_data: T,
    ) -> crate::Result<SendRawTransactionResp>
    where
        T: serde::Serialize + Debug,
    {
        self.do_request::<_, SendRawTransactionResp>("wallet/broadcasttransaction", Some(raw_data))
            .await
    }

    // 获取链参数
    pub async fn chain_params(&self) -> crate::Result<ChainParameter> {
        Ok(self.client.get_request("wallet/getchainparameters").await?)
    }

    // trx transfer fee ,check to address exist
    pub async fn transfer_fee(
        &self,
        account: &str,
        to: Option<&str>,
        tx: &RawTransactionParams,
        signature_num: u8,
    ) -> crate::Result<ResourceConsumer> {
        let chain_params = self.chain_params().await?;
        let resource = self.account_resource(account).await?;

        let mut consumer = if let Some(to) = to {
            // check to address exist
            let to_account = self.account_info(to).await?;

            if !to_account.address.is_empty() {
                let bandwidth = self.calc_bandwidth(&tx.raw_data_hex, signature_num);
                let bandwidth = Resource::new(
                    resource.available_bandwidth(),
                    bandwidth,
                    chain_params.get_transaction_fee(),
                    "bandwidth",
                );
                ResourceConsumer::new(bandwidth, None)
            } else {
                let consumer = chain_params.get_create_account_transfer_fee();
                // convert to bandwidth
                let consumer = consumer / chain_params.get_transaction_fee();

                let bandwidth = Resource::new(
                    resource.available_stake_bandwidth(),
                    consumer,
                    chain_params.get_transaction_fee(),
                    "bandwidth",
                );

                let mut resource = ResourceConsumer::new(bandwidth, None);
                resource.set_extra_fee(chain_params.get_create_account());
                resource
            }
        } else {
            let bandwidth = self.calc_bandwidth(&tx.raw_data_hex, signature_num);
            let bandwidth = Resource::new(
                resource.available_bandwidth(),
                bandwidth,
                chain_params.get_transaction_fee(),
                "bandwidth",
            );
            ResourceConsumer::new(bandwidth, None)
        };
        if signature_num > 1 {
            consumer.set_extra_fee(chain_params.get_multi_sign_fee());
        }

        Ok(consumer)
    }

    // calculate contract fee
    pub async fn contract_fee<T>(
        &self,
        params: super::operations::contract::ConstantContract<T>,
        signature_num: u8,
        account: &str,
    ) -> crate::Result<ResourceConsumer> {
        let bandwidth = self.calc_bandwidth(&params.transaction.raw_data_hex, signature_num);

        // six bytes for fee_limit
        let bandwidth = bandwidth + 6;

        let resource = self.account_resource(account).await?;
        let chain_params = self.chain_params().await?;

        let bandwidth = Resource::new(
            resource.available_bandwidth(),
            bandwidth,
            chain_params.get_transaction_fee(),
            "bandwidth",
        );

        let energy = Resource::new(
            resource.available_energy(),
            params.energy_used as i64,
            chain_params.get_energy_fee(),
            "energy",
        );

        let mut consumer = ResourceConsumer::new(bandwidth, Some(energy));
        if signature_num > 1 {
            consumer.set_extra_fee(chain_params.get_multi_sign_fee());
        }

        Ok(consumer)
    }

    // 计算交易要使用多少宽带(字节数)
    pub fn calc_bandwidth(&self, raw_data_hex: &str, signature_num: u8) -> i64 {
        let data_hex_pro = 3_i64;
        let result_hex = 64_i64;
        let sign_len = 67_i64 * signature_num as i64;

        let raw_data_len = (raw_data_hex.len() / 2) as i64;
        raw_data_len + data_hex_pro + result_hex + sign_len
    }
}

// new version
pub struct TronProvider {
    client: HttpClient,
}
impl TronProvider {
    pub fn new(rpc_url: &str) -> crate::Result<Self> {
        let client = HttpClient::new(rpc_url, None)?;
        Ok(Self { client })
    }

    pub async fn chain_params(&self) -> crate::Result<ChainParameter> {
        Ok(self.client.get("wallet/getchainparameters").send().await?)
    }

    pub async fn account_info(&self, account: &str) -> crate::Result<TronAccount> {
        let mut params = HashMap::new();
        params.insert("address", account);
        if account.starts_with("T") {
            params.insert("visible", "true");
        }

        let res = self
            .client
            .post_request("wallet/getaccount", params)
            .await?;
        Ok(res)
    }

    pub async fn account_resource(&self, account: &str) -> crate::Result<AccountResourceDetail> {
        let mut params = HashMap::new();
        params.insert("address", account);
        if account.starts_with("T") {
            params.insert("visible", "true");
        }

        let res = self
            .client
            .post_request("wallet/getaccountresource", params)
            .await?;
        Ok(res)
    }

    pub async fn query_tx_info(&self, tx_hash: &str) -> crate::Result<TransactionInfo> {
        let mut params = HashMap::new();
        params.insert("value", tx_hash);

        let result = self
            .client
            .post("wallet/gettransactioninfobyid")
            .json(params)
            .send::<TransactionInfo>()
            .await?;
        Ok(result)
    }

    pub async fn send_raw_transaction(
        &self,
        params: SendRawTransactionParams,
    ) -> crate::Result<SendRawTransactionResp> {
        let result = self
            .client
            .post_request("wallet/broadcasttransaction", params)
            .await?;
        Ok(result)
    }

    pub async fn can_delegate_resource(
        &self,
        owner_address: &str,
        _resource: ResourceType,
    ) -> crate::Result<String> {
        let mut params = HashMap::new();

        let owner_address = wallet_utils::address::bs58_addr_to_hex(owner_address)?;
        params.insert("owner_address", json!(owner_address));
        params.insert("type", json!(0));
        // params.insert("type", resource.to_int_str());

        let result = self
            .client
            .post_request("wallet/getcandelegatedmaxsize", params)
            .await?;
        Ok(result)
    }

    // build base transfer
    pub async fn create_base_transfer(
        &self,
        params: BaseTransaction,
    ) -> crate::Result<CreateTransactionResp<BaseTransaction>> {
        let res = self
            .client
            .post_request("wallet/createtransaction", params)
            .await?;
        Ok(res)
    }

    pub fn calc_bandwidth(&self, raw_data_hex: &str, signature_num: u8) -> i64 {
        let data_hex_pro = 3_i64;
        let result_hex = 64_i64;
        let sign_len = 67_i64 * signature_num as i64;

        let raw_data_len = (raw_data_hex.len() / 2) as i64;
        raw_data_len + data_hex_pro + result_hex + sign_len
    }

    pub async fn freeze_balance(
        &self,
        args: params::FreezeBalanceArgs,
    ) -> crate::Result<CreateTransactionResp<FreezeBalanceResp>> {
        let res = self
            .client
            .post_request("wallet/freezebalancev2", args)
            .await?;
        Ok(res)
    }

    pub async fn unfreeze_balance(
        &self,
        args: params::UnFreezeBalanceArgs,
    ) -> crate::Result<CreateTransactionResp<UnFreezeBalanceResp>> {
        let res = self
            .client
            .post_request("wallet/unfreezebalancev2", args)
            .await?;
        Ok(res)
    }

    pub async fn delegate_resource(
        &self,
        args: params::DelegateArgs,
    ) -> crate::Result<CreateTransactionResp<DelegateResp>> {
        let res = self
            .client
            .post_request("wallet/delegateresource", args)
            .await?;
        Ok(res)
    }

    pub async fn un_delegate_resource(
        &self,
        args: params::UnDelegateArgs,
    ) -> crate::Result<CreateTransactionResp<UnDelegateResp>> {
        let res = self
            .client
            .post_request("wallet/undelegateresource", args)
            .await?;
        Ok(res)
    }

    pub async fn withdraw_expire_unfree(
        &self,
        owner_address: &str,
    ) -> crate::Result<CreateTransactionResp<WithdrawExpire>> {
        let mut args = HashMap::new();
        let owner_address = wallet_utils::address::bs58_addr_to_hex(owner_address)?;
        args.insert("owner_address", owner_address);
        let res = self
            .client
            .post_request("wallet/withdrawexpireunfreeze", args)
            .await?;
        Ok(res)
    }

    pub async fn can_withdraw_unfreeze_amount(
        &self,
        owner_address: &str,
    ) -> crate::Result<CanWithdrawUnfreezeAmount> {
        let owner_address = wallet_utils::address::bs58_addr_to_hex(owner_address)?;
        let mut args = HashMap::new();
        args.insert("owner_address", owner_address);
        let res = self
            .client
            .post_request("wallet/getcanwithdrawunfreezeamount", args)
            .await?;
        Ok(res)
    }
}
// // 主币相关的交易错误类型
// #[derive(Debug, serde::Deserialize)]
// struct NodeError {
//     pub code: String,
//     pub message: String,
// }

// // 不是合约的调用、匹配成功和失败的情况
// async fn do_normal_request<T, R>(&self, endpoint: &str, params: Option<T>) -> crate::Result<R>
// where
//     T: serde::Serialize + Debug,
//     R: serde::de::DeserializeOwned,
// {
//     let request = self.client.post(endpoint);
//     let request = if let Some(params) = params {
//         request.json(&params)
//     } else {
//         request
//     };

//     match request.send::<NodeResponse<R, ContractError>>().await? {
//         NodeResponse::Success(r) => Ok(r),
//         NodeResponse::Fail(err) => {
//             let error_msg = hex_func::hex_to_utf8(&err.result.message)?;
//             Err(crate::Error::RpcError(error_msg))
//         }
//     }
// }

#[test]
fn test_aaa() {
    let s = r#"{"result":{"result":true},"transaction":{"visible":false,"txID":"eb2d58896b86c7b65ab007131ff21d237933be9d2834c33215d4ac768e0e48d2","raw_data":{"contract":[{"parameter":{"value":{"data":"a9059cbb000000000000000000000000fe26169e8a994ceb9addd4b8fb3a8f28d134e30500000000000000000000000000000000000000000000000000000000000186a0","owner_address":"414cec0660c26bcd7f33795e97b96e9fc27e17d8af","contract_address":"41ea51342dabbb928ae1e576bd39eff8aaf070a8c6"},"type_url":"type.googleapis.com/protocol.TriggerSmartContract"},"type":"TriggerSmartContract"}],"ref_block_bytes":"9f16","ref_block_hash":"8d8c6b82f6308136","expiration":1729161357000,"fee_limit":2045190,"timestamp":1729161298385},"raw_data_hex":"0a029f1622088d8c6b82f630813640c8cdb1d0a9325aae01081f12a9010a31747970652e676f6f676c65617069732e636f6d2f70726f746f636f6c2e54726967676572536d617274436f6e747261637412740a15414cec0660c26bcd7f33795e97b96e9fc27e17d8af121541ea51342dabbb928ae1e576bd39eff8aaf070a8c62244a9059cbb000000000000000000000000fe26169e8a994ceb9addd4b8fb3a8f28d134e30500000000000000000000000000000000000000000000000000000000000186a070d183aed0a932900186ea7c"}}"#;

    let res = serde_json::from_str::<
        NodeResponse<TriggerContractResult<ContractTransferResp>, ContractError>,
    >(s)
    .unwrap();

    println!("{:?}", res);
}
