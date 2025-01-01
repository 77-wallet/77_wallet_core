use super::consts::TRX_TO_SUN;

#[derive(Debug)]
pub struct Resource {
    // user resource
    pub limit: i64,
    // tx consumer resource
    pub consumer: i64,
    // unit is sun
    pub price: i64,
    // energy or bandwidth
    pub types: String,
}
impl Resource {
    pub fn new(limit: i64, consumer: i64, price: i64, types: &str) -> Self {
        Self {
            limit,
            consumer,
            price,
            types: types.to_string(),
        }
    }

    // 对于带宽来说，用户的免费带宽是小于交易所需要的带宽，那么这种情况，直接燃烧交易需要的带宽对应的trx
    pub fn need_extra_resource(&self) -> i64 {
        if self.types == "bandwidth" {
            if self.consumer > self.limit {
                self.consumer
            } else {
                0
            }
        } else if self.consumer > self.limit {
            self.consumer - self.limit
        } else {
            0
        }
    }

    pub fn fee(&self) -> i64 {
        self.price * self.need_extra_resource()
    }
}

#[derive(Debug)]
pub struct ResourceConsumer {
    pub energy: Option<Resource>,
    pub bandwidth: Resource,
    // unit is sun
    pub extra_fee: i64,
}

impl ResourceConsumer {
    pub fn new(bandwidth: Resource, energy: Option<Resource>) -> Self {
        Self {
            energy,
            bandwidth,
            extra_fee: 0,
        }
    }

    // unit is sun
    pub fn set_extra_fee(&mut self, extra_fee: i64) {
        self.extra_fee += extra_fee;
    }

    pub fn transaction_fee(&self) -> String {
        let bandwidth_fee = self.bandwidth.fee();
        let energy_fee = if let Some(energy) = self.energy.as_ref() {
            energy.fee()
        } else {
            0
        };

        let total = bandwidth_fee + energy_fee + self.extra_fee;

        (total as f64 / TRX_TO_SUN as f64).to_string()
    }

    // unit is sun
    pub fn transaction_fee_i64(&self) -> i64 {
        let bandwidth_fee = self.bandwidth.fee();
        let energy_fee = if let Some(energy) = self.energy.as_ref() {
            energy.fee()
        } else {
            0
        };

        bandwidth_fee + energy_fee + self.extra_fee
    }

    // 用于合约交易时设置fee_limit(不考虑用户用户的资源)
    pub fn fee_limit(&self) -> i64 {
        let mut fee = self.bandwidth.consumer * self.bandwidth.price;

        if let Some(energy) = &self.energy {
            fee += energy.consumer * energy.price;
        }

        fee
    }

    pub fn get_energy(&self) -> u64 {
        if let Some(energy) = &self.energy {
            energy.consumer as u64
        } else {
            0
        }
    }

    pub fn need_extra_energy(&self) -> i64 {
        if let Some(energy) = self.energy.as_ref() {
            energy.need_extra_resource()
        } else {
            0
        }
    }

    pub fn need_extra_bandwidth(&self) -> i64 {
        self.bandwidth.need_extra_resource()
    }

    // 一笔交易实际使用的能量
    pub fn act_energy(&self) -> i64 {
        self.energy
            .as_ref()
            .map_or(0, |energy| energy.consumer.min(energy.limit))
    }

    pub fn act_bandwidth(&self) -> i64 {
        if self.bandwidth.consumer > self.bandwidth.limit {
            0
        } else {
            self.bandwidth.consumer
        }
    }
}
