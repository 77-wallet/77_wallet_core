use super::{
    operations::{
        TronTransactionResponse,
        contract::{ConstantContract, TriggerContractParameter, TriggerContractResult},
        stake::{self, CanDelegatedMaxSize},
        transfer::{ContractTransferResp, TronTransferResp},
    },
    params::ResourceConsumer,
    protocol::{
        account::{AccountResourceDetail, TronAccount},
        block::TronBlock,
        chain_parameter::ChainParameter,
        receipt::TransactionInfo,
        transaction::SendRawTransactionResp,
    },
};
use crate::tron::params::Resource;
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

// 直接调用api错误
#[derive(Debug, serde::Deserialize)]
struct ApiError {
    #[allow(unused)]
    code: String,
    message: String,
}
// 波场验证错误
#[derive(Debug, serde::Deserialize)]
struct ContractValidateException {
    #[serde(rename = "Error")]
    error: crate::ContractValidationError,
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

    // 调用波场api
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

        let response_str = request.do_request().await?;

        // 处理结果
        match wallet_utils::serde_func::serde_from_str::<R>(&response_str) {
            Ok(res) => Ok(res),
            Err(_) => {
                // 反序列化失败后，尝试解析错误。
                let result = wallet_utils::serde_func::serde_from_str::<ApiError>(&response_str);

                let result = match result {
                    Ok(res) => {
                        let error_msg = hex_func::hex_to_utf8(&res.message)?;
                        crate::Error::RpcError(error_msg)
                    }
                    Err(_e) => {
                        let result = wallet_utils::serde_func::serde_from_str::<
                            ContractValidateException,
                        >(&response_str)?;
                        crate::Error::ContractValidationError(result.error)
                    }
                };
                Err(result)
            }
        }
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
        let params = Some(json!({"detail":false}));
        self.do_request::<_, TronBlock>("wallet/getblock", params)
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
        raw_data_hex: &str,
        signature_num: u8,
    ) -> crate::Result<ResourceConsumer> {
        let chain_params = self.chain_params().await?;
        let resource = self.account_resource(account).await?;

        let mut consumer = if let Some(to) = to {
            // check to address exist
            let to_account = self.account_info(to).await?;

            if !to_account.address.is_empty() {
                let bandwidth = self.calc_bandwidth(raw_data_hex, signature_num);
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
            let bandwidth = self.calc_bandwidth(raw_data_hex, signature_num);
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

// abount stake and degegate
impl Provider {
    pub async fn can_delegate_resource(
        &self,
        owner_address: &str,
        resource: stake::ResourceType,
    ) -> crate::Result<CanDelegatedMaxSize> {
        let owner_address = wallet_utils::address::bs58_addr_to_hex(owner_address)?;

        let args = json!({
            "owner_address":json!(owner_address),
            "type":json!(resource.to_i8())
        });

        let result = self
            .do_request("wallet/getcandelegatedmaxsize", Some(args))
            .await?;
        Ok(result)
    }

    pub async fn freeze_balance(
        &self,
        args: &stake::FreezeBalanceArgs,
    ) -> crate::Result<TronTransactionResponse<stake::FreezeBalanceResp>> {
        let res = self
            .do_request("wallet/freezebalancev2", Some(args))
            .await?;
        Ok(res)
    }

    pub async fn unfreeze_balance(
        &self,
        args: &stake::UnFreezeBalanceArgs,
    ) -> crate::Result<TronTransactionResponse<stake::UnFreezeBalanceResp>> {
        let res = self
            .do_request("wallet/unfreezebalancev2", Some(args))
            .await?;
        Ok(res)
    }

    pub async fn cancel_all_unfreeze(
        &self,
        args: &stake::CancelAllFreezeBalanceArgs,
    ) -> crate::Result<TronTransactionResponse<stake::CancelAllUnfreezeResp>> {
        let res = self
            .do_request("wallet/cancelallunfreezev2", Some(args))
            .await?;
        Ok(res)
    }

    pub async fn delegate_resource(
        &self,
        args: &stake::DelegateArgs,
    ) -> crate::Result<TronTransactionResponse<stake::DelegateResp>> {
        let res = self
            .do_request("wallet/delegateresource", Some(args))
            .await?;
        Ok(res)
    }

    pub async fn un_delegate_resource(
        &self,
        args: &stake::UnDelegateArgs,
    ) -> crate::Result<TronTransactionResponse<stake::UnDelegateResp>> {
        let res = self
            .do_request("wallet/undelegateresource", Some(args))
            .await?;
        Ok(res)
    }

    pub async fn withdraw_expire_unfree(
        &self,
        owner_address: &str,
        permission_id: Option<i64>,
    ) -> crate::Result<TronTransactionResponse<stake::WithdrawExpireResp>> {
        let owner_address = wallet_utils::address::bs58_addr_to_hex(owner_address)?;

        let args = if let Some(permision_id) = permission_id {
            json!({
                "owner_address":json!(owner_address),
                "Permission_id":json!(permision_id),
            })
        } else {
            json!({
                "owner_address":json!(owner_address),
            })
        };

        // let args = json!({
        //     "owner_address":json!(owner_address)
        // });
        let res = self
            .do_request("wallet/withdrawexpireunfreeze", Some(args))
            .await?;

        Ok(res)
    }

    // available
    pub async fn can_withdraw_unfreeze_amount(
        &self,
        owner_address: &str,
    ) -> crate::Result<stake::CanWithdrawUnfreezeAmount> {
        let owner_address = wallet_utils::address::bs58_addr_to_hex(owner_address)?;

        let args = json!({
            "owner_address":json!(owner_address)
        });

        let res = self
            .do_request("wallet/getcanwithdrawunfreezeamount", Some(args))
            .await?;
        Ok(res)
    }

    // query the resource delegation index by an account. Two lists will return, one is the list of addresses the account has delegated its resources(toAddress), and the other is the list of addresses that have delegated resources to the account(fromAddress).
    pub async fn delegate_others_list(&self, owner: &str) -> crate::Result<stake::DelegateOther> {
        let args = json!({
            "value":json!(owner),
            "visible":json!(true),
        });

        let res = self
            .do_request("wallet/getdelegatedresourceaccountindexv2", Some(args))
            .await?;
        Ok(res)
    }

    // query the detail of resource share delegated from fromAddress to toAddress
    pub async fn delegated_resource(
        &self,
        owner: &str,
        to: &str,
    ) -> crate::Result<stake::DelegatedResource> {
        let args = json!({
            "fromAddress":json!(owner),
            "toAddress":json!(to),
            "visible":json!(true),
        });
        let res = self
            .do_request("wallet/getdelegatedresourcev2", Some(args))
            .await?;
        Ok(res)
    }

    pub async fn get_reward(&self, owner: &str) -> crate::Result<stake::Reward> {
        let args = json!({
            "address":json!(owner),
            "visible":json!(true),
        });
        let res = self.do_request("wallet/getReward", Some(args)).await?;
        Ok(res)
    }

    // 领取投票的奖励
    pub async fn withdraw_balance(
        &self,
        owner_address: &str,
        permission_id: Option<i64>,
    ) -> crate::Result<TronTransactionResponse<stake::WithdrawBalanceResp>> {
        let owner_address = wallet_utils::address::bs58_addr_to_hex(owner_address)?;

        let args = if let Some(permission) = permission_id {
            json!({
                "owner_address":json!(owner_address),
                "Permission_id":json!(permission)
            })
        } else {
            json!({
                "owner_address":json!(owner_address),
            })
        };

        let res = self
            .do_request("wallet/withdrawbalance", Some(args))
            .await?;

        Ok(res)
    }

    pub async fn votes_wintess(
        &self,
        args: &stake::VoteWitnessArgs,
    ) -> crate::Result<TronTransactionResponse<stake::VoteWitnessResp>> {
        let res = self
            .do_request("wallet/votewitnessaccount", Some(args))
            .await?;
        Ok(res)
    }

    pub async fn list_witnesses(&self) -> crate::Result<stake::ListWitnessResp> {
        let res = self.do_request("wallet/listwitnesses", None::<()>).await?;
        Ok(res)
    }

    pub async fn get_brokerage(&self, address: &str) -> crate::Result<stake::BrokerageResp> {
        let args = json!({
            "address": address,
        });
        let res = self.do_request("wallet/getBrokerage", Some(args)).await?;
        Ok(res)
    }
}
