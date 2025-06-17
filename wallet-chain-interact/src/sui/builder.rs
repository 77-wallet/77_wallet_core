use super::{Provider, consts::SUI_NATIVE_COIN};
use sui_json_rpc_types::Coin;
use sui_types::{
    base_types::ObjectRef,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Argument, Command, ObjectArg},
};

pub struct SelectCoinHelper {
    // 选择的coin
    pub select_coins: Vec<Coin>,
    // gas_coin
    pub gas_coin: Vec<Coin>,
}

impl SelectCoinHelper {
    // 选择代币的总金额
    pub fn select_total_amount(&self) -> u64 {
        self.select_coins.iter().map(|c| c.balance).sum()
    }

    pub fn gas_amount(&self) -> u64 {
        self.gas_coin.iter().map(|c| c.balance).sum()
    }

    // 选择代币和gas的总金额
    pub fn select_total_with_gas(&self) -> u64 {
        self.select_total_amount() + self.gas_coin.iter().map(|c| c.balance).sum::<u64>()
    }

    // true 需要单独一个对象支付gas false 不需要
    pub fn need_extra_coin_pay_gas(&self, coin_type: &str, gas_fee: u64, amount: u64) -> bool {
        let check_amount = if coin_type == SUI_NATIVE_COIN {
            gas_fee + amount
        } else {
            gas_fee
        };

        let gas_amount = self.gas_coin.iter().map(|c| c.balance).sum::<u64>();

        gas_amount < check_amount
    }

    // 增加gas coin into select coin
    // notes select coin must be main coin
    pub fn add_gas_coin(&mut self, gas_coin: Vec<Coin>) {
        self.gas_coin.extend(gas_coin);
    }

    // 提出到已经选择的coin,在选择其他sui币作为手续费
    pub async fn select_gas_coin(
        &self,
        gas_fee: u64,
        provider: &Provider,
        owner: &str,
    ) -> crate::Result<Vec<Coin>> {
        let coin = provider
            .get_all_coins_by_owner(owner, SUI_NATIVE_COIN)
            .await?;

        let unselect_coin = coin
            .into_iter()
            .filter(|c| !self.select_coins.contains(c))
            .collect::<Vec<Coin>>();

        // 加上原来的gas
        let mut sum = self.gas_amount();
        let mut gas_coin = vec![];
        for coin in unselect_coin {
            sum += coin.balance;
            gas_coin.push(coin);
            if sum >= gas_fee {
                break;
            }
        }
        if sum < gas_fee {
            return Err(crate::sui::error::SuiError::InsufficientFee(sum, gas_fee).into());
        }

        Ok(gas_coin)
    }

    pub fn gas_obj_ref(&self) -> Vec<ObjectRef> {
        self.gas_coin.iter().map(|c| c.object_ref()).collect()
    }

    pub fn build_main_coin_arg(
        &mut self,
        builder: &mut ProgrammableTransactionBuilder,
        amount: u64,
        gas_fee: u64,
    ) -> crate::Result<Vec<Argument>> {
        // 是否进行拆分币(绝大多情况下是需要的)
        if self.select_total_with_gas() > gas_fee + amount {
            let pure_amount = builder
                .pure(amount)
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            // tracing::warn!("split gas coin");

            Ok(vec![
                builder.command(Command::SplitCoins(Argument::GasCoin, vec![pure_amount])),
            ])
        } else {
            // tracing::warn!("select total with gas: {}", self.select_total_with_gas());
            // tracing::warn!("gas fee: {}", gas_fee + amount);
            // tracing::warn!("not split gas coin");

            Ok(vec![Argument::GasCoin])
        }
    }

    // 代币转账构建转账的arg
    pub fn build_token_coin_arg(
        &mut self,
        builder: &mut ProgrammableTransactionBuilder,
        amount: u64,
    ) -> crate::Result<Vec<Argument>> {
        // 代币
        let mut coin_inputs = vec![];
        for coin in self.select_coins.iter() {
            let coin_arg = builder
                .obj(ObjectArg::ImmOrOwnedObject(coin.object_ref()))
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            coin_inputs.push(coin_arg);
        }

        // merge_coin
        let primary = coin_inputs[0];
        if coin_inputs.len() > 1 {
            // tracing::warn!("merge coin");
            builder.command(Command::MergeCoins(primary, coin_inputs[1..].to_vec()));
        }

        //
        let transfer_amount = if self.select_total_amount() > amount {
            // tracing::warn!("split coins");
            let pure_amount = builder
                .pure(amount)
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;

            vec![builder.command(Command::SplitCoins(primary, vec![pure_amount]))]
        } else {
            vec![primary]
        };

        Ok(transfer_amount)
    }
}
