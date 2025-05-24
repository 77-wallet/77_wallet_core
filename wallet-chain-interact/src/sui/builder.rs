use super::{Provider, consts::SUI_NATIVE_COIN};
use sui_json_rpc_types::Coin;
use sui_types::{
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Argument, Command, ObjectArg},
};

pub type SelectedCoins = Vec<Coin>;

pub struct SelectCoinHelper {
    // 用户一次查询的coin
    pub all_coins: Vec<Coin>,
    // 选择的coin
    pub select_coins: SelectedCoins,
}

impl SelectCoinHelper {
    pub async fn new() -> Self {
        SelectCoinHelper {
            all_coins: vec![],
            select_coins: vec![],
        }
    }

    // 选择coin
    pub async fn select_coins(
        amount: u64,
        provider: &Provider,
        coin_type: &str,
        addr: &str,
    ) -> crate::Result<Self> {
        let mut all_coin = provider.get_all_coins_by_owner(addr, coin_type).await?;
        all_coin.sort_by_key(|c| std::cmp::Reverse(c.balance));

        let mut selected = vec![];

        let mut sum = 0u64;
        for coin in &all_coin {
            sum += coin.balance;
            selected.push(coin.clone());
            if sum >= amount {
                break;
            }
        }

        if sum < amount {
            return Err(crate::Error::SuiError(
                crate::sui::error::SuiError::InsufficientBalance(sum, amount),
            ));
        }

        Ok(SelectCoinHelper {
            all_coins: all_coin,
            select_coins: selected,
        })
    }

    pub fn select_total_amount(&self) -> u64 {
        self.select_coins.iter().map(|c| c.balance).sum()
    }

    // true 需要单独一个对象支付gas false 不需要
    pub fn need_extra_coin_pay_gas(&self, coin_type: &str, gas_fee: u64, amount: u64) -> bool {
        if coin_type == SUI_NATIVE_COIN {
            //  所选择的金额是否大于 fee + 转账金额
            return self.select_total_amount() < gas_fee + amount;
        }

        true
    }

    pub fn base_trans(
        &self,
        builder: &mut ProgrammableTransactionBuilder,
        amount: u64,
    ) -> crate::Result<Vec<Argument>> {
        // input
        let mut coin_inputs = vec![];
        let mut total = 0u64;

        for coin in self.select_coins.iter() {
            let coin_arg = builder
                .obj(ObjectArg::ImmOrOwnedObject(coin.object_ref()))
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;
            coin_inputs.push(coin_arg);
            total += coin.balance;
        }
        let primary = coin_inputs[0];
        if coin_inputs.len() > 1 {
            tracing::warn!("merge coins");
            builder.command(Command::MergeCoins(primary, coin_inputs[1..].to_vec()));
        }

        // 是否需要拆分
        let amount_arg = if total > amount {
            tracing::warn!("split coins");
            let pure_amount = builder
                .pure(amount)
                .map_err(|e| crate::sui::error::SuiError::MoveError(e.to_string()))?;

            builder.command(Command::SplitCoins(primary, vec![pure_amount]))
        } else {
            primary
        };

        Ok(vec![amount_arg])
    }

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

        let mut sum = 0u64;
        let mut gas_coin = vec![];
        for c in unselect_coin {
            sum += c.balance;
            gas_coin.push(c);
            if sum >= gas_fee {
                break;
            }
        }

        Ok(gas_coin)
    }

    // 是否需要拆分
}
