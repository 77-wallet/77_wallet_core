use super::SuiBaseTransaction;
use crate::{
    sui::{
        Provider,
        builder::SelectCoinHelper,
        consts::SUI_NATIVE_COIN,
        protocol::{EstimateFeeResp, GasObject},
    },
    types,
};
use alloy::primitives::U256;
use std::str::FromStr;
use sui_types::{
    Identifier, TypeTag,
    base_types::{ObjectRef, SuiAddress},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Argument, Command, ObjectArg, ProgrammableTransaction, TransactionData},
};
use wallet_utils::address;

pub struct TransferOpt {
    pub from: SuiAddress,
    pub to: SuiAddress,
    pub amount: u64,
    pub token: Option<String>,
}

impl TransferOpt {
    pub fn new(from: &str, to: &str, amount: U256, token: Option<String>) -> crate::Result<Self> {
        Ok(Self {
            from: address::parse_sui_address(from)?,
            to: address::parse_sui_address(to)?,
            amount: amount.to::<u64>(),
            token,
        })
    }

    pub fn get_coin_type(&self) -> String {
        self.token.clone().unwrap_or(SUI_NATIVE_COIN.to_string())
    }

    pub async fn build_pt(
        &self,
        provider: &Provider,
    ) -> crate::Result<(ProgrammableTransaction, SelectCoinHelper)> {
        let mut builder = ProgrammableTransactionBuilder::new();

        let from = self.from.to_string();

        let coin_type = self.get_coin_type();
        let select_helper =
            SelectCoinHelper::select_coins(self.amount, provider, &coin_type, &from).await?;

        let amount_arg = select_helper.base_trans(&mut builder, self.amount)?;

        // command
        let receipt = builder
            .pure(self.to)
            .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
        builder.command(Command::TransferObjects(amount_arg, receipt));

        Ok((builder.finish(), select_helper))
    }

    pub async fn build_data(
        &self,
        provider: &Provider,
        helper: SelectCoinHelper,
        gas: EstimateFeeResp,
    ) -> crate::Result<TransactionData> {
        let mut builder = ProgrammableTransactionBuilder::new();

        let coin_type = self.get_coin_type();
        let from = self.from.to_string();

        let gas_fee = gas.get_fee();

        tracing::warn!("select coins: {:#?}", helper.select_coins);

        //  是否需要一额外引入一个对象做gas 费用
        let (amount_arg, gas_obj) =
            if helper.need_extra_coin_pay_gas(&coin_type, gas_fee, self.amount) {
                tracing::warn!("need extra gas coin");
                let amount = helper.base_trans(&mut builder, self.amount)?;
                let gas_coin = helper.select_gas_coin(gas_fee, provider, &from).await?;

                (amount, GasObject::new(gas_coin))
            } else {
                // 第一个是最大的coin 拿出来支付gas
                let gas_coin = helper.select_coins[0].clone();
                let gas_obj = GasObject::new(vec![gas_coin.clone()]);

                // 只选择了一个,一个币既要用来支付gas 也需要支付业务转账
                // 选择了两个,第一个币既要用来支付gas 也要支付业务转账 ,剩余的币合并在gas_coin 里面
                let amount_arg = if helper.select_coins.len() == 1 {
                    tracing::warn!("only one coin");

                    // 第一个币拆分出转账的金额
                    let amount = builder
                        .pure(self.amount)
                        .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
                    let amount_arg =
                        builder.command(Command::SplitCoins(Argument::GasCoin, vec![amount]));

                    vec![amount_arg]
                } else {
                    // 第一个币拆分出来 gas_fee 后剩余的金额
                    tracing::warn!("选择的币数量{:?}", helper.select_coins);
                    tracing::warn!("转账金额{}", self.amount);

                    // 第一个币拆分出 gas_fee 剩余的部分作为转账的金额
                    let remain = helper.select_coins[0].balance - gas_fee;
                    let mut remain_amount = self.amount - remain;

                    tracing::warn!(
                        "拆出来的金额{} 第一个币的余额{}",
                        remain,
                        helper.select_coins[0].balance
                    );
                    tracing::warn!("手续费{}", gas_fee);
                    let remain_arg = builder
                        .pure(remain)
                        .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
                    let remain_arg =
                        builder.command(Command::SplitCoins(Argument::GasCoin, vec![remain_arg]));

                    let mut amount_org = vec![remain_arg];

                    for coin in helper.select_coins.iter().skip(1) {
                        if remain_amount == 0 {
                            break;
                        }

                        let coin_arg = builder
                            .obj(ObjectArg::ImmOrOwnedObject(coin.object_ref()))
                            .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;

                        // 判断当前币是否 需要拆分
                        if coin.balance > remain_amount {
                            tracing::warn!("need split coin");

                            tracing::warn!(
                                "当前币的余额{},拆分金额{}",
                                coin.balance,
                                remain_amount
                            );

                            let pure_amount = builder.pure(remain_amount).map_err(|e| {
                                crate::sui::error::SuiError::MoveError(e.to_string())
                            })?;
                            let arg =
                                builder.command(Command::SplitCoins(coin_arg, vec![pure_amount]));
                            remain_amount = 0;

                            amount_org.push(arg);
                        } else {
                            amount_org.push(coin_arg);
                            remain_amount -= coin.balance;
                        }
                    }

                    amount_org
                };

                (amount_arg, gas_obj)
            };

        //  转账的command
        let receipt = builder
            .pure(self.to)
            .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
        builder.command(Command::TransferObjects(amount_arg, receipt));

        // 选择gas对象
        let gas_payment = gas_obj
            .coin
            .iter()
            .map(|f| f.object_ref())
            .collect::<Vec<_>>();

        let tx_data = TransactionData::new_programmable(
            self.from,
            gas_payment,
            builder.finish(),
            gas_fee,
            gas.gas_price,
        );

        Ok(tx_data)
    }
}

pub struct TransferOpt2 {
    pub base: SuiBaseTransaction,
    pub struct_tag: Option<TypeTag>,
}

impl TransferOpt2 {
    pub fn new(
        from: &str,
        to: &str,
        amount: u64,
        transfer_coins: Vec<ObjectRef>,
        gas_coins: Vec<ObjectRef>,
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
            .map(|struct_tag| address::parse_sui_type_tag(struct_tag))
            .transpose()?;

        Ok(Self { base, struct_tag })
    }
}

impl types::Transaction<TransactionData> for TransferOpt2 {
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
            tracing::info!("amount: {}", self.base.amount);
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

            // SuiStructTag
            // TypeTag::

            let coin_struct_tag_json = serde_json::json!(
                {
                    "address": sui_types::SUI_FRAMEWORK_ADDRESS,
                    "module": Identifier::new("coin").unwrap(),
                    "name": Identifier::new("Coin").unwrap(),
                    // "type_args": vec![TypeTag::Struct(Box::new(struct_tag.clone()))],
                    "type_args": vec![struct_tag.clone()],
                }
            );

            let b = wallet_utils::serde_func::serde_from_value(coin_struct_tag_json)?;
            // let coin_struct_tag = sui_sdk::StructTag {
            //     address: ,
            //     module: Identifier::new("coin").unwrap(),
            //     name: Identifier::new("Coin").unwrap(),
            //     type_params: vec![TypeTag::Struct(Box::new(struct_tag.clone()))],
            // };

            builder.command(Command::move_call(
                address::parse_object_id_from_hex("0x2")?,
                module,
                function,
                // vec![TypeTag::Struct(Box::new(struct_tag.clone()))],
                vec![TypeTag::Struct(Box::new(b))],
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
