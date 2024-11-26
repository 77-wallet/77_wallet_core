#[derive(Debug)]
pub struct SolFeeSetting {
    // unit lamports
    pub base_fee: u64,
    // unit lamports
    pub priority_fee_per_compute_unit: Option<u64>,
    // consumed compute units
    pub compute_units_consumed: u64,
    // extra_fee unit is lamports
    pub extra_fee: Option<u64>,
}

impl SolFeeSetting {
    pub fn new(base_fee: u64, compute_units: u64) -> Self {
        Self {
            base_fee,
            priority_fee_per_compute_unit: None,
            compute_units_consumed: compute_units,
            extra_fee: None,
        }
    }
}

impl SolFeeSetting {
    // unit is sol
    pub fn transaction_fee(&self) -> f64 {
        let fee = self.original_fee() as f64;
        fee / super::consts::SOL_VALUE as f64
    }

    // unit is lamports
    pub fn original_fee(&self) -> u64 {
        // priority fee
        let priority = if let Some(priority) = self.priority_fee_per_compute_unit {
            self.compute_units_consumed * priority
        } else {
            0
        };

        let extra_fee = self.extra_fee.unwrap_or(0);

        self.base_fee + priority + extra_fee
    }
}
