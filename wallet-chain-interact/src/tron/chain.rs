use super::operations::contract::TriggerContractParameter;
use super::operations::{self, RawData, RawTransactionParams};
use super::params::ResourceConsumer;
use super::protocol::account::{AccountResourceDetail, TronAccount};
use super::provider::Provider;
use crate::tron::protocol::protobuf::transaction::Raw;
use crate::types::{ChainPrivateKey, MultisigTxResp};
use crate::QueryTransactionResult;
use alloy::primitives::U256;
use std::fmt::Debug;
use wallet_utils::{serde_func, sign};

pub struct TronChain {
    pub provider: Provider,
}
impl TronChain {
    pub fn new(provider: Provider) -> crate::Result<Self> {
        Ok(Self { provider })
    }

    pub fn get_provider(&self) -> &Provider {
        &self.provider
    }
}

impl TronChain {
    pub async fn balance(&self, addr: &str, token: Option<String>) -> crate::Result<U256> {
        if let Some(t) = token {
            let trigger = TriggerContractParameter::token_balance_trigger(&t, addr)?;
            let result = self.provider.trigger_constant_contract(trigger).await?;
            result.parse_u256()
        } else {
            let account = self.provider.account_info(addr).await?;
            Ok(U256::from(account.balance))
        }
    }

    pub async fn block_num(&self) -> crate::Result<u64> {
        let block_height = self.provider.get_block().await?;
        Ok(block_height.block_header.raw_data.number)
    }

    pub async fn decimals(&self, token: &str) -> crate::Result<u8> {
        let trigger = TriggerContractParameter::decimal_trigger(token)?;

        let res = self.provider.trigger_constant_contract(trigger).await?;
        let value = res.parse_u256()?;

        Ok(value.to::<u8>())
    }

    pub async fn token_symbol(&self, token: &str) -> crate::Result<String> {
        let trigger = TriggerContractParameter::symbol_trigger(token)?;

        let res = self.provider.trigger_constant_contract(trigger).await?;
        let value = res.parse_string()?;
        Ok(value.chars().filter(|c| c.is_alphanumeric()).collect())
    }

    pub async fn token_name(&self, token: &str) -> crate::Result<String> {
        let trigger = TriggerContractParameter::name_trigger(token)?;

        let res = self.provider.trigger_constant_contract(trigger).await?;
        let value = res.parse_string()?;

        Ok(value.chars().filter(|c| c.is_alphanumeric()).collect())
    }

    pub async fn black_address(&self, token: &str, owner: &str) -> crate::Result<bool> {
        let trigger = TriggerContractParameter::black_address(token, owner)?;

        let res = self.provider.trigger_constant_contract(trigger).await?;

        res.parse_bool()
    }

    pub async fn account_resource(&self, account: &str) -> crate::Result<AccountResourceDetail> {
        self.provider.account_resource(account).await
    }

    pub async fn account_info(&self, account: &str) -> crate::Result<TronAccount> {
        self.provider.account_info(account).await
    }

    // 内部构件交易原始数据
    pub async fn exec_transaction<T, R>(
        &self,
        params: T,
        key: ChainPrivateKey,
    ) -> crate::Result<String>
    where
        T: operations::TronTxOperation<R>,
        R: serde::Serialize + Debug,
    {
        let mut raw = params.build_raw_transaction(&self.provider).await?;

        let sign = sign::sign_tron(&raw.tx_id, &key, None)?;
        raw.signature.push(sign);

        let res = self.provider.exec_raw_transaction(raw).await?;

        Ok(res.tx_id)
    }

    // 外部来构建交易数据
    pub async fn exec_transaction_v1(
        &self,
        mut raw_transaction: RawTransactionParams,
        key: ChainPrivateKey,
    ) -> crate::Result<String> {
        let sign = sign::sign_tron(&raw_transaction.tx_id, &key, None)?;
        raw_transaction.signature.push(sign);

        let result = self.provider.exec_raw_transaction(raw_transaction).await?;
        Ok(result.tx_id)
    }

    pub async fn build_multisig_transaction<T, R>(
        &self,
        params: T,
        expiration: u64,
    ) -> crate::Result<MultisigTxResp>
    where
        T: operations::TronTxOperation<R>,
        R: serde::Serialize + Debug + serde::de::DeserializeOwned,
    {
        let mut resp = params.build_raw_transaction(&self.provider).await?;

        let mut raw_data = serde_func::serde_from_str::<RawData<R>>(&resp.raw_data)?;

        // expiration unit is ms
        let new_time = raw_data.expiration + expiration * 1000;
        raw_data.expiration = new_time;

        let mut raw = Raw::from_str(&resp.raw_data_hex)?;
        raw.expiration = new_time as i64;

        let bytes = raw.to_bytes()?;

        resp.tx_id = Raw::tx_id(&bytes);
        resp.raw_data_hex = Raw::raw_data_hex(&bytes);
        resp.raw_data = raw_data.to_json_string()?;

        Ok(MultisigTxResp {
            tx_hash: resp.tx_id.clone(),
            raw_data: resp.to_string()?,
        })
    }

    // trx fee: this method is estimate fee by create transaction
    pub async fn simple_fee<T, R>(
        &self,
        account: &str,
        signature_num: u8,
        params: T,
    ) -> crate::Result<ResourceConsumer>
    where
        T: operations::TronTxOperation<R>,
        R: serde::Serialize + Debug,
    {
        let tx = params.build_raw_transaction(&self.provider).await?;
        let to = params.get_to();
        let to = if to.is_empty() {
            None
        } else {
            Some(to.as_str())
        };

        self.provider
            .transfer_fee(account, to, &tx.raw_data_hex, signature_num)
            .await
    }

    // trx fee : this method is estimate fee by simulate a transaction
    pub async fn simulate_simple_fee<T>(
        &self,
        account: &str,
        to: &str,
        signature_num: u8,
        params: T,
    ) -> crate::Result<ResourceConsumer>
    where
        T: operations::TronSimulateOperation,
    {
        let raw_data_hex = params.simulate_raw_transaction()?;

        let to = if to.is_empty() { None } else { Some(to) };

        self.provider
            .transfer_fee(account, to, &raw_data_hex, signature_num)
            .await
    }

    // contract fee
    pub async fn contract_fee<T, R>(
        &self,
        account: &str,
        signature_num: u8,
        params: T,
    ) -> crate::Result<ResourceConsumer>
    where
        T: operations::TronConstantOperation<R>,
        R: serde::Serialize + Debug,
    {
        let raw = params.constant_contract(&self.provider).await?;

        self.provider
            .contract_fee(raw, signature_num, account)
            .await
    }

    pub async fn exec_multisig_transaction(
        &self,
        mut params: RawTransactionParams,
        sign_seq: Vec<String>,
    ) -> crate::Result<String> {
        params.signature = sign_seq;
        let res = self.provider.exec_raw_transaction(params).await?;
        Ok(res.tx_id)
    }

    // 查询交易结果
    pub async fn query_tx_res(&self, hash: &str) -> crate::Result<Option<QueryTransactionResult>> {
        let transaction = self.provider.query_tx_info(hash).await;
        let transaction = match transaction {
            Ok(transaction) => transaction,
            Err(_err) => return Ok(None),
        };

        // timestamp unit ms to s
        // let time = (transaction.block_timestamp / 1000) - (8 * 3600);
        let time = transaction.block_timestamp / 1000;
        let fee = transaction.fee / super::consts::TRX_TO_SUN as f64;
        let status = if transaction.result.is_none() { 2 } else { 3 };

        let resource_consume = transaction
            .receipt
            .get_bill_resource_consumer()
            .to_json_str()?;
        Ok(Some(QueryTransactionResult::new(
            transaction.id,
            fee,
            resource_consume,
            time,
            status,
            transaction.block_number,
        )))
    }

    // pub async fn withdraw_unfreeze_amount(
    //     &self,
    //     owner_address: &str,
    //     key: ChainPrivateKey,
    // ) -> crate::Result<String> {
    //     let resp = self.provider.withdraw_expire_unfree(owner_address).await?;

    //     let raw = RawTransactionParams::from(resp);

    //     // signature
    //     let sign_str = wallet_utils::sign::sign_tron(&raw.tx_id, &key, None)?;

    //     let rs = self.exec_multisig_transaction(raw, vec![sign_str]).await?;
    //     Ok(rs)
    // }
}
