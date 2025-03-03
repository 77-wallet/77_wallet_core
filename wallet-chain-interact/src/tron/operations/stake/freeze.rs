use super::ResourceType;
use crate::tron::{
    consts,
    operations::{RawTransactionParams, TronTxOperation},
    Provider,
};

#[derive(serde::Serialize, Debug)]
pub struct FreezeBalanceArgs {
    owner_address: String,
    resource: ResourceType,
    frozen_balance: i64,
    visible: bool,
    #[serde(rename = "Permission_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_id: Option<i64>,
}

impl FreezeBalanceArgs {
    pub fn new(
        owner_address: &str,
        resource: &str,
        frozen_balance: i64,
        permission_id: Option<i64>,
    ) -> crate::Result<Self> {
        Ok(Self {
            owner_address: wallet_utils::address::bs58_addr_to_hex(owner_address)?,
            resource: ResourceType::try_from(resource)?,
            frozen_balance: frozen_balance * consts::TRX_VALUE,
            visible: false,
            permission_id,
        })
    }
}

#[async_trait::async_trait]
impl TronTxOperation<FreezeBalanceResp> for FreezeBalanceArgs {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams> {
        let res = provider.freeze_balance(self).await?;
        Ok(RawTransactionParams::from(res))
    }

    fn get_to(&self) -> String {
        String::new()
    }

    fn get_value(&self) -> f64 {
        (self.frozen_balance / consts::TRX_VALUE) as f64
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct FreezeBalanceResp {
    #[serde(skip_serializing_if = "Option::is_none")]
    resource: Option<String>,
    frozen_balance: i64,
    owner_address: String,
}
