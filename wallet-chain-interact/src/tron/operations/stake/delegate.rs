use super::ResourceType;
use crate::tron::consts;
use crate::tron::operations::{RawTransactionParams, TronTxOperation};
use crate::tron::provider::Provider;

#[derive(serde::Serialize, Debug)]
pub struct DelegateArgs {
    pub owner_address: String,
    pub receiver_address: String,
    pub balance: u64,
    pub resource: ResourceType,
    pub lock: bool,
    pub lock_period: i64,
}
impl DelegateArgs {
    pub fn new(
        owner_address: &str,
        receiver_address: &str,
        balance: &str,
        resource: &str,
    ) -> crate::Result<Self> {
        let balance = wallet_utils::unit::convert_to_u256(balance, consts::TRX_DECIMALS)?;
        Ok(Self {
            owner_address: wallet_utils::address::bs58_addr_to_hex(owner_address)?,
            receiver_address: wallet_utils::address::bs58_addr_to_hex(receiver_address)?,
            balance: balance.to::<u64>(),
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
        String::new()
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, serde::Deserialize)]
pub struct CanWithdrawUnfreezeAmount {
    #[serde(default)]
    pub amount: i64,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WithdrawExpire {
    owner_address: String,
}
