use super::ResourceType;
use crate::tron::{
    consts,
    operations::{RawTransactionParams, TronSimulateOperation, TronTxOperation},
    Provider,
};
use anychain_core::Transaction;
use anychain_tron::protocol::{balance_contract::UnDelegateResourceContract, common::ResourceCode};
use protobuf::EnumOrUnknown;

#[derive(Debug, serde::Serialize)]
pub struct UnDelegateArgs {
    pub owner_address: String,
    pub receiver_address: String,
    pub balance: i64,
    pub resource: ResourceType,
}
impl UnDelegateArgs {
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
        self.receiver_address.clone()
    }

    fn get_value(&self) -> f64 {
        (self.balance / consts::TRX_VALUE) as f64
    }
}

impl TronSimulateOperation for UnDelegateArgs {
    fn simulate_raw_transaction(&self) -> crate::Result<String> {
        let mut dr_contract = UnDelegateResourceContract::new();

        let resource_code = match self.resource {
            ResourceType::BANDWIDTH => ResourceCode::BANDWIDTH,
            ResourceType::ENERGY => ResourceCode::ENERGY,
        };

        dr_contract.owner_address = wallet_utils::hex_func::hex_decode(&self.owner_address)?;
        dr_contract.receiver_address = wallet_utils::hex_func::hex_decode(&self.receiver_address)?;
        dr_contract.balance = self.balance;
        dr_contract.resource = EnumOrUnknown::<ResourceCode>::new(resource_code);
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
pub struct UnDelegateResp {
    owner_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource: Option<String>,
    receiver_address: String,
    balance: i64,
}
