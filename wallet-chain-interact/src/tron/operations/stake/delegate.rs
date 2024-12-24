use super::ResourceType;
use crate::tron::consts;
use crate::tron::operations::{RawTransactionParams, TronSimulateOperation, TronTxOperation};
use crate::tron::provider::Provider;
use anychain_core::Transaction as _;
use anychain_tron::protocol::balance_contract::DelegateResourceContract;
use anychain_tron::protocol::common::ResourceCode;
use protobuf::EnumOrUnknown;

#[derive(serde::Serialize, Debug)]
pub struct DelegateArgs {
    pub owner_address: String,
    pub receiver_address: String,
    pub balance: i64,
    pub resource: ResourceType,
    pub lock: bool,
    pub lock_period: i64,
}
impl DelegateArgs {
    pub fn new(
        owner_address: &str,
        receiver_address: &str,
        balance: i64,
        resource: &str,
    ) -> crate::Result<Self> {
        Ok(Self {
            owner_address: wallet_utils::address::bs58_addr_to_hex(owner_address)?,
            receiver_address: wallet_utils::address::bs58_addr_to_hex(receiver_address)?,
            balance: balance * consts::TRX_VALUE,
            resource: ResourceType::try_from(resource)?,
            lock: false,
            lock_period: 0,
        })
    }

    pub fn with_lock_period(mut self, lock_period: i64) -> Self {
        self.lock = true;
        self.lock_period = lock_period;
        self
    }
}

#[async_trait::async_trait]
impl TronTxOperation<DelegateResp> for DelegateArgs {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams> {
        let res = provider.delegate_resource(self).await?;
        Ok(RawTransactionParams::from(res))
    }

    fn get_to(&self) -> String {
        self.receiver_address.clone()
    }
}

impl TronSimulateOperation for DelegateArgs {
    fn simulate_raw_transaction(&self) -> crate::Result<String> {
        let mut dr_contract = DelegateResourceContract::new();

        let resource_code = match self.resource {
            ResourceType::BANDWIDTH => ResourceCode::BANDWIDTH,
            ResourceType::ENERGY => ResourceCode::ENERGY,
        };

        dr_contract.owner_address = wallet_utils::hex_func::hex_decode(&self.owner_address)?;
        dr_contract.receiver_address = wallet_utils::hex_func::hex_decode(&self.receiver_address)?;
        dr_contract.balance = self.balance;
        dr_contract.resource = EnumOrUnknown::<ResourceCode>::new(resource_code);
        dr_contract.lock = self.lock;
        dr_contract.lock_period = self.lock_period;
        let ct = anychain_tron::trx::build_contract(&dr_contract)?;

        let mut param = anychain_tron::TronTransactionParameters::default();
        param.set_timestamp(anychain_tron::trx::timestamp_millis());
        param.set_ref_block(Self::DEFAULT_NUM, Self::DEFAULT_HASH);
        param.set_contract(ct);

        let transaction = anychain_tron::TronTransaction::new(&param)?;

        let raw_data_hex = transaction.to_bytes()?;
        Ok(wallet_utils::hex_func::hex_encode(raw_data_hex))
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct DelegateResp {
    owner_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource: Option<String>,
    receiver_address: String,
    balance: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    lock: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lock_period: Option<i64>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegateOther {
    pub account: String,
    #[serde(default)]
    pub from_accounts: Vec<String>,
    #[serde(default)]
    pub to_accounts: Vec<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DelegatedResource {
    #[serde(rename = "delegatedResource")]
    pub delegated_resource: Vec<DelegateResouce>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DelegateResouce {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub frozen_balance_for_bandwidth: i64,
    #[serde(default)]
    pub frozen_balance_for_energy: i64,
    #[serde(default)]
    pub expire_time_for_bandwidth: i64,
    #[serde(default)]
    pub expire_time_for_energy: i64,
}
impl DelegateResouce {
    pub fn value_trx(&self, resource_type: ResourceType) -> i64 {
        match resource_type {
            ResourceType::BANDWIDTH => {
                self.frozen_balance_for_bandwidth / consts::TRX_TO_SUN as i64
            }
            ResourceType::ENERGY => self.frozen_balance_for_energy / consts::TRX_TO_SUN as i64,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct CanWithdrawUnfreezeAmount {
    #[serde(default)]
    pub amount: i64,
}
impl CanWithdrawUnfreezeAmount {
    pub fn to_sun(&self) -> i64 {
        self.amount / consts::TRX_VALUE
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct WithdrawExpire {
    owner_address: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct CanDelegatedMaxSize {
    #[serde(default)]
    // unit is sun
    pub max_size: i64,
}

impl CanDelegatedMaxSize {
    pub fn to_sun(&self) -> i64 {
        self.max_size / consts::TRX_VALUE
    }
}
