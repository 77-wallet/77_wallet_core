use std::str::FromStr;

use crate::types;
use move_core_types::language_storage::StructTag;
use sui_sdk::types::transaction::TransactionData;
use sui_types::{
    base_types::ObjectRef,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Command, ProgrammableMoveCall},
    Identifier, TypeTag,
};
use wallet_utils::address;

use super::SuiBaseTransaction;

pub struct TransferOpt {
    pub base: SuiBaseTransaction,
    pub struct_tag: Option<StructTag>,
}

impl TransferOpt {
    pub fn new(
        from: &str,
        // recipients: Vec<&str>,
        to: &str,
        amount: u64,
        transfer_coins: Vec<ObjectRef>,
        gas_coins: Vec<ObjectRef>,
        // gas_budget: u64,
        // gas_price: u64,
        struct_tag: Option<String>,
    ) -> crate::Result<Self> {
        let base = SuiBaseTransaction::new(
            from, // recipients,
            to,
            amount, // input_coins,
            transfer_coins,
            gas_coins, // gas_budget, gas_price,
        )?;

        let struct_tag = struct_tag
            .as_ref()
            .map(|struct_tag| address::parse_sui_struct_tag(struct_tag))
            .transpose()?;

        Ok(Self { base, struct_tag })
    }
}

impl types::Transaction<TransactionData> for TransferOpt {
    fn build_transaction(&self) -> Result<TransactionData, crate::Error> {
        let mut builder = ProgrammableTransactionBuilder::new();

        // let primary_ref = self.base.transfer_coins[0];
        let mut coin_inputs = vec![];
        for coin in &self.base.transfer_coins {
            let coin_arg = builder
                .obj(sui_types::transaction::ObjectArg::ImmOrOwnedObject(*coin))
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            coin_inputs.push(coin_arg);
        }

        let primary = coin_inputs[0];
        let to_merge = &coin_inputs[1..];

        if !to_merge.is_empty() {
            builder.command(Command::MergeCoins(primary, to_merge.to_vec()));
        }

        if let Some(struct_tag) = &self.struct_tag {
            // 合约调用交易（类似 ERC20 转账）
            // 构造纯数据参数
            // let amount_arg = sui_types::transaction::CallArg::Pure(
            //     wallet_utils::serde_func::bcs_to_bytes(&self.base.amount)?,
            // );
            // builder.split_coin(self.base.to, primary_ref, vec![self.base.amount]);
            let split_amount = builder
                .pure(self.base.amount /* u64 */)
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            // let split_arg = builder
            //     .split_coin(primary_arg.clone(), vec![amount_arg])
            //     .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;

            let split_result = builder.command(Command::SplitCoins(primary, vec![split_amount]));
            let sui_types::transaction::Argument::Result(index) = split_result else {
                return Err(crate::sui::error::SuiError::MoveError(
                    "SplitCoins did not return Result".into(),
                )
                .into());
            };
            let split_coin_arg = sui_types::transaction::Argument::NestedResult(index, 0);

            // let coins_arg = sui_types::transaction::CallArg::Object(
            //     sui_types::transaction::ObjectArg::ImmOrOwnedObject(split_ref),
            // );
            // let to_arg = sui_types::transaction::CallArg::Pure(
            //     wallet_utils::serde_func::bcs_to_bytes(&self.base.to)?,
            // );

            let to_arg = builder
                .pure(self.base.to)
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;

            let module: Identifier = Identifier::from_str("transfer")
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            let function: Identifier = Identifier::from_str("public_transfer")
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            // struct_tag.

            let coin_struct_tag = StructTag {
                address: move_core_types::account_address::AccountAddress::from_str("0x2").unwrap(),
                module: Identifier::new("coin").unwrap(),
                name: Identifier::new("Coin").unwrap(),
                type_params: vec![TypeTag::Struct(Box::new(struct_tag.clone()))],
            };

            builder.command(Command::move_call(
                address::parse_object_id_from_hex("0x2")?,
                module,
                function,
                // vec![TypeTag::Struct(Box::new(struct_tag.clone()))],
                vec![TypeTag::Struct(Box::new(coin_struct_tag))],
                vec![split_coin_arg, to_arg],
            ));
            // builder
            //     .move_call(
            //         address::parse_object_id_from_hex("0x2")?,
            //         // struct_tag.address.into(),
            //         module,
            //         function,
            //         vec![TypeTag::Struct(Box::new(struct_tag.clone()))],
            //         vec![coins_arg, to_arg],
            //     )
            //     .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            let pt = builder.finish();
            Ok(TransactionData::new_programmable(
                self.base.from,
                self.base.gas_coins.clone(),
                pt,
                10_000_000,
                10000,
            ))
        } else {
            // 3. Split 出需要转账的金额（如果不等于 coin 全额）
            let pure_amount = builder
                .pure(self.base.amount)
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            let result = builder.command(Command::SplitCoins(primary, vec![pure_amount]));

            // 4. Transfer 刚才 split 出的金额（结果索引为上一步的第一个 result）
            let p = builder
                .pure(self.base.to)
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            builder.command(Command::TransferObjects(vec![result], p));
            // // 合并 coin
            // builder
            //     .merge_coins(primary_ref, self.base.coin_refs[1..].to_vec())
            //     .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            // builder.split_coin(self.base.to, primary_ref, vec![self.base.amount]);

            // builder.transfer_sui(self.base.to, Some(self.base.amount));
            let pt = builder.finish();
            // 原生 SUI 转账
            Ok(TransactionData::new_programmable(
                self.base.from,
                self.base.gas_coins.clone(), // Gas 对象
                pt,
                10_000_000,
                10000,
            ))
        }
    }
}
