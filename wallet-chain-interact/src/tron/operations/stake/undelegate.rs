use crate::tron::{
    operations::{RawTransactionParams, TronTxOperation},
    Provider,
};

use super::ResourceType;

#[derive(Debug, serde::Serialize)]
pub struct UnDelegateArgs {
    pub owner_address: String,
    pub receiver_address: String,
    pub balance: u64,
    pub resource: ResourceType,
}
impl UnDelegateArgs {
    pub fn new(
        owner_address: &str,
        receiver_address: &str,
        balance: &str,
        resource: &str,
    ) -> crate::Result<Self> {
        let balance = wallet_utils::unit::convert_to_u256(balance, 6)?;

        Ok(Self {
            owner_address: wallet_utils::address::bs58_addr_to_hex(owner_address)?,
            receiver_address: wallet_utils::address::bs58_addr_to_hex(receiver_address)?,
            balance: balance.to::<u64>(),
            resource: ResourceType::try_from(resource)?,
        })
    }
}

#[async_trait::async_trait]
impl TronTxOperation<UnDelegateResp> for UnDelegateArgs {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams> {
        let res = provider.un_delegate_resource(self).await?;
        Ok(RawTransactionParams::from(res))
    }

    fn get_to(&self) -> String {
        String::new()
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UnDelegateResp {
    owner_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource: Option<String>,
    receiver_address: String,
    balance: i64,
}
