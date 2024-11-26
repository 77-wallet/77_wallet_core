use super::EthereumBaseTransaction;
use crate::{eth::protocol::contract::transferCall, types};
use alloy::{
    network::TransactionBuilder, primitives, rpc::types::TransactionRequest, sol_types::SolCall,
};
use wallet_utils::address;

pub struct TransferOpt {
    pub base: EthereumBaseTransaction,
    pub contract: Option<primitives::Address>,
}

impl TransferOpt {
    pub fn new(
        from: &str,
        to: &str,
        value: primitives::U256,
        contract: Option<String>,
    ) -> crate::Result<Self> {
        let base = EthereumBaseTransaction::new(from, to, value)?;

        let contract = contract
            .as_ref()
            .map(|contract| address::parse_eth_address(contract))
            .transpose()?;

        Ok(Self { base, contract })
    }
}

impl types::Transaction<TransactionRequest> for TransferOpt {
    fn build_transaction(&self) -> Result<TransactionRequest, crate::Error> {
        if let Some(contract) = self.contract {
            let call = transferCall {
                from: self.base.to,
                amount: self.base.value,
            };
            Ok(TransactionRequest::default()
                .from(self.base.from)
                .to(contract)
                .value(primitives::U256::ZERO)
                .with_input(call.abi_encode()))
        } else {
            Ok(TransactionRequest::default()
                .from(self.base.from)
                .to(self.base.to)
                .value(self.base.value))
        }
    }
}
