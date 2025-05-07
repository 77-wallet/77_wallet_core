use crate::types;
use sui_sdk::types::{
    base_types::{ObjectID, SuiAddress},
    transaction::TransactionData,
};
use wallet_utils::address;

use super::SuiBaseTransaction;

pub struct TransferOpt {
    pub base: SuiBaseTransaction,
    pub contract: Option<SuiAddress>,
    pub gas_object: ObjectID,
}

impl TransferOpt {
    pub fn new(
        from: &str,
        to: &str,
        value: u128,
        contract: Option<String>,
        gas_object: &str,
    ) -> crate::Result<Self> {
        let base = SuiBaseTransaction::new(from, to, value)?;

        let contract = contract
            .as_ref()
            .map(|contract| address::parse_sui_address(contract))
            .transpose()?;

        Ok(Self {
            base,
            contract,
            gas_object: ObjectID::from_hex(gas_object)?,
        })
    }
}

impl types::Transaction<TransactionData> for TransferOpt {
    fn build_transaction(&self) -> Result<TransactionData, crate::Error> {
        if let Some(contract) = self.contract {
            // 合约调用交易（类似 ERC20 转账）
            let mut builder =
                sui_programmable_transaction_builder::ProgrammableTransactionBuilder::new();
            builder.move_call(
                contract,
                "transfer".parse()?,
                vec![],
                vec![self.base.value.into(), self.base.to.into()],
            );
            let pt = builder.finish();

            Ok(TransactionData::new_programmable(
                self.base.from,
                vec![self.gas_object.into()], // Gas 对象
                pt,
                self.base.gas_budget,
                self.base.gas_price,
            ))
        } else {
            // 原生 SUI 转账
            Ok(TransactionData::new_transfer(
                self.base.to,
                self.base.value,
                self.base.from,
                self.gas_object.into(), // Gas 对象
                self.base.gas_budget,
                self.base.gas_price,
            ))
        }
    }
}
