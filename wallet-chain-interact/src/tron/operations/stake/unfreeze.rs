use super::{ResourceType, WithdrawExpire};
use crate::tron::{
    operations::{RawTransactionParams, TronTxOperation},
    Provider,
};

#[derive(serde::Serialize, Debug)]
pub struct UnFreezeBalanceArgs {
    owner_address: String,
    resource: ResourceType,
    unfreeze_balance: i64,
}

impl UnFreezeBalanceArgs {
    pub fn new(owner_address: &str, resource: &str, unfreeze_balance: &str) -> crate::Result<Self> {
        let unfreeze_balance = wallet_utils::unit::convert_to_u256(unfreeze_balance, 6)?;
        Ok(Self {
            owner_address: wallet_utils::address::bs58_addr_to_hex(owner_address)?,
            resource: ResourceType::try_from(resource)?,
            unfreeze_balance: unfreeze_balance.to::<i64>(),
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
}
impl CancelAllFreezeBalanceArgs {
    pub fn new(owner_address: &str) -> crate::Result<Self> {
        Ok(Self {
            owner_address: wallet_utils::address::bs58_addr_to_hex(owner_address)?,
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
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct CancelAllUnfreezeResp {
    owner_address: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct WithdrawUnfreezeArgs {
    pub owner_address: String,
}
#[async_trait::async_trait]
impl TronTxOperation<WithdrawExpire> for WithdrawUnfreezeArgs {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams> {
        let res = provider.withdraw_expire_unfree(&self.owner_address).await?;
        Ok(RawTransactionParams::from(res))
    }

    fn get_to(&self) -> String {
        String::new()
    }
}
