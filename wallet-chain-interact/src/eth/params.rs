use alloy::primitives::U256;

#[derive(Default, Debug)]
pub struct FeeSetting {
    pub base_fee: U256,
    pub max_priority_fee_per_gas: U256,
    pub max_fee_per_gas: U256,
    pub gas_limit: U256,
}

impl FeeSetting {
    pub fn transaction_fee(&self) -> U256 {
        let price = self.base_fee + self.max_priority_fee_per_gas;
        // let price = self.max_fee_per_gas;
        self.gas_limit * price
    }

    pub fn new_with_price(price: U256) -> Self {
        let priority_fee = U256::from(2_000_000_000u64);
        let max_fee = price + priority_fee;

        Self {
            base_fee: price,
            gas_limit: U256::from(21000),
            max_priority_fee_per_gas: priority_fee,
            max_fee_per_gas: max_fee,
        }
    }
}

#[derive(Debug)]
pub struct EtherFee {
    pub base_fee: U256,
    pub priority_fee_per_gas: U256,
}
