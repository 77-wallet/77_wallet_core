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
        // let price = self.base_fee + self.max_priority_fee_per_gas;
        let price = self.max_fee_per_gas;
        self.gas_limit * price
    }
}

#[derive(Debug)]
pub struct EtherFee {
    pub base_fee: U256,
    pub priority_fee_per_gas: U256,
}
