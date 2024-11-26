use alloy::primitives::U256;

// 交易手续费计算的结果
#[derive(Debug, Default)]
pub struct FeeResponse {
    pub gas_price: U256,
    pub gas_used: U256,
    pub energy_used: U256,
    pub energy_price: U256,
    pub fee: Option<U256>,
}
impl FeeResponse {
    pub fn new(gas_price: U256, gas_used: U256) -> Self {
        Self {
            gas_price,
            gas_used,
            energy_used: U256::ZERO,
            energy_price: U256::ZERO,
            fee: None,
        }
    }
    pub fn energy_price(mut self, energy_price: U256) -> Self {
        self.energy_price = energy_price;
        self
    }
    pub fn energy_used(mut self, energy_used: U256) -> Self {
        self.energy_used = energy_used;
        self
    }

    pub fn set_fee(mut self, fee: U256) -> Self {
        self.fee = Some(fee);
        self
    }

    pub fn calc_fee(&self, unit: u8) -> Result<String, wallet_utils::error::Error> {
        let rs = self.gas_price * self.gas_used + self.energy_price * self.energy_used;
        wallet_utils::unit::format_to_string(rs, unit)
    }
    pub fn calc_fee_original(&self) -> Result<U256, wallet_utils::error::Error> {
        Ok(self.gas_price * self.gas_used + self.energy_price * self.energy_used)
    }
}

#[derive(Default, Debug)]
pub struct QueryTransactionResult {
    pub hash: String,
    // main unit like btc ,sol,eth
    pub transaction_fee: f64,
    pub resource_consume: String,
    pub transaction_time: u128,
    // 2success 3fail
    pub status: i8,
    pub block_height: u128,
}
impl QueryTransactionResult {
    pub fn new(
        hash: String,
        transaction_fee: f64,
        resource_consume: String,
        transaction_time: u128,
        status: i8,
        block_height: u128,
    ) -> Self {
        Self {
            hash,
            transaction_fee,
            resource_consume,
            transaction_time,
            status,
            block_height,
        }
    }
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct BillResourceConsume {
    pub net_used: u64,
    pub energy_used: u64,
}
impl BillResourceConsume {
    pub fn new_tron(net_used: u64, energy_used: u64) -> Self {
        Self {
            net_used,
            energy_used,
        }
    }

    pub fn one_resource(net_used: u64) -> Self {
        Self {
            net_used,
            energy_used: 0,
        }
    }

    pub fn to_json_str(&self) -> crate::Result<String> {
        Ok(wallet_utils::serde_func::serde_to_string(&self)?)
    }

    pub fn from_json_str(s: &str) -> crate::Result<Self> {
        Ok(wallet_utils::serde_func::serde_from_str(s)?)
    }
}

#[derive(serde::Serialize, Debug)]
pub struct ResourceConsume {
    pub consume: i64,
}
impl ResourceConsume {
    pub fn new(consume: i64) -> Self {
        Self { consume }
    }
}
