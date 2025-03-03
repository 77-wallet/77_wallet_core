use super::ResourceType;
use crate::tron::{
    consts,
    operations::{RawTransactionParams, TronTxOperation},
    Provider,
};

#[derive(serde::Serialize, Debug)]
pub struct UnFreezeBalanceArgs {
    owner_address: String,
    resource: ResourceType,
    unfreeze_balance: i64,
    #[serde(rename = "Permission_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_id: Option<i64>,
}

impl UnFreezeBalanceArgs {
    pub fn new(
        owner_address: &str,
        resource: &str,
        unfreeze_balance: i64,
        permission_id: Option<i64>,
    ) -> crate::Result<Self> {
        Ok(Self {
            owner_address: wallet_utils::address::bs58_addr_to_hex(owner_address)?,
            resource: ResourceType::try_from(resource)?,
            unfreeze_balance: unfreeze_balance * consts::TRX_VALUE,
            permission_id,
        })
    }
}

#[async_trait::async_trait]
impl TronTxOperation<UnFreezeBalanceResp> for UnFreezeBalanceArgs {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams> {
        let res = provider.unfreeze_balance(self).await?;
        Ok(RawTransactionParams::from(res))
    }

    fn get_to(&self) -> String {
        String::new()
    }

    fn get_value(&self) -> f64 {
        (self.unfreeze_balance / consts::TRX_VALUE) as f64
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct UnFreezeBalanceResp {
    #[serde(skip_serializing_if = "Option::is_none")]
    resource: Option<String>,
    unfreeze_balance: i64,
    owner_address: String,
}

#[derive(serde::Serialize, Debug)]
pub struct CancelAllFreezeBalanceArgs {
    owner_address: String,
    #[serde(rename = "Permission_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_id: Option<i64>,
}
impl CancelAllFreezeBalanceArgs {
    pub fn new(owner_address: &str, permission_id: Option<i64>) -> crate::Result<Self> {
        Ok(Self {
            owner_address: wallet_utils::address::bs58_addr_to_hex(owner_address)?,
            permission_id,
        })
    }
}

#[async_trait::async_trait]
impl TronTxOperation<CancelAllUnfreezeResp> for CancelAllFreezeBalanceArgs {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams> {
        let res = provider.cancel_all_unfreeze(self).await?;
        Ok(RawTransactionParams::from(res))
    }

    fn get_to(&self) -> String {
        String::new()
    }
    fn get_value(&self) -> f64 {
        0.0
    }
}
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct CancelAllUnfreezeResp {
    owner_address: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct WithdrawUnfreezeArgs {
    pub owner_address: String,
    #[serde(rename = "Permission_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_id: Option<i64>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct WithdrawExpireResp {
    owner_address: String,
}
#[async_trait::async_trait]
impl TronTxOperation<WithdrawExpireResp> for WithdrawUnfreezeArgs {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams> {
        let res = provider
            .withdraw_expire_unfree(&self.owner_address, self.permission_id)
            .await?;
        Ok(RawTransactionParams::from(res))
    }

    fn get_to(&self) -> String {
        String::new()
    }

    fn get_value(&self) -> f64 {
        0.0
    }
}
