use std::str::FromStr;

use crate::types;
use sui_sdk::types::{
    base_types::{ObjectID, SuiAddress},
    transaction::TransactionData,
};
use sui_types::{
    base_types::ObjectRef, programmable_transaction_builder::ProgrammableTransactionBuilder,
    Identifier,
};
use wallet_utils::address;

use super::SuiBaseTransaction;

pub struct TransferOpt {
    pub base: SuiBaseTransaction,
    pub contract: Option<SuiAddress>,
}

impl TransferOpt {
    pub fn new(
        from: &str,
        to: &str,
        amount: u128,
        transfer_object_ref: ObjectRef,
        gas_payment_ref: ObjectRef,
        gas_budget: u64,
        gas_price: u64,
        contract: Option<String>,
    ) -> crate::Result<Self> {
        let base = SuiBaseTransaction::new(
            from,
            to,
            amount,
            transfer_object_ref,
            gas_payment_ref,
            gas_budget,
            gas_price,
        )?;

        let contract = contract
            .as_ref()
            .map(|contract| address::parse_sui_address(contract))
            .transpose()?;

        Ok(Self { base, contract })
    }
}

impl types::Transaction<TransactionData> for TransferOpt {
    fn build_transaction(&self) -> Result<TransactionData, crate::Error> {
        if let Some(contract) = self.contract {
            // 合约调用交易（类似 ERC20 转账）
            let mut builder = ProgrammableTransactionBuilder::new();
            // 构造纯数据参数
            let amount_arg = sui_types::transaction::CallArg::Pure(
                wallet_utils::serde_func::bcs_to_bytes(&self.base.amount)?,
            );
            let to_arg = sui_types::transaction::CallArg::Pure(
                wallet_utils::serde_func::bcs_to_bytes(&self.base.to)?,
            );

            let module: Identifier = Identifier::from_str("coin")
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            let function: Identifier = Identifier::from_str("transfer")
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;

            builder
                .move_call(
                    contract.into(),
                    module,
                    function,
                    vec![],
                    vec![amount_arg, to_arg],
                )
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            let pt = builder.finish();

            Ok(TransactionData::new_programmable(
                self.base.from,
                vec![self.base.gas_payment_ref.clone()], // Gas 对象
                pt,
                self.base.gas_budget,
                self.base.gas_price,
            ))
        } else {
            // 原生 SUI 转账
            Ok(TransactionData::new_transfer(
                self.base.to,
                // 转账对象
                self.base.transfer_object_ref.clone(),
                // sender
                self.base.from,
                // gas payment
                self.base.gas_payment_ref.clone(),
                self.base.gas_budget,
                self.base.gas_price,
            ))
        }
    }
}
