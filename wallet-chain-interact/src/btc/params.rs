#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct FeeSetting {
    pub fee_rate: bitcoin::Amount,
    pub size: usize,
}
impl FeeSetting {
    // unit is btc
    pub fn transaction_fee(&self) -> String {
        let res = self.fee_rate * self.size as u64;
        let rs = res.to_float_in(bitcoin::Denomination::Bitcoin);
        rs.to_string()
    }

    // unit is btc f64
    pub fn transaction_fee_f64(&self) -> f64 {
        let res = self.fee_rate * self.size as u64;
        res.to_float_in(bitcoin::Denomination::Bitcoin)
    }
}
