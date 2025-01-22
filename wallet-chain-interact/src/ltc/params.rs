#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct FeeSetting {
    pub fee_rate: litecoin::Amount,
    pub size: usize,
}
impl FeeSetting {
    // unit is btc
    pub fn transaction_fee(&self) -> String {
        let res = self.fee_rate * self.size as u64;
        let rs = res.to_float_in(litecoin::Denomination::Bitcoin);
        rs.to_string()
    }

    // unit is btc f64
    pub fn transaction_fee_f64(&self) -> f64 {
        let res = self.fee_rate * self.size as u64;
        res.to_float_in(litecoin::Denomination::Bitcoin)
    }
}

#[derive(Debug)]
pub struct TransferResp {
    pub tx_hash: String,
    pub fee: f64,
}
impl TransferResp {
    pub fn new(tx_hash: String, fee_rate: litecoin::Amount, size: usize) -> Self {
        let fee = (fee_rate * size as u64).to_float_in(litecoin::Denomination::Bitcoin);
        Self { tx_hash, fee }
    }
}
