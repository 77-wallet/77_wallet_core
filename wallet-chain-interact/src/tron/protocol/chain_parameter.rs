use crate::tron::consts::TRX_TO_SUN;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ParameterValue {
    pub key: String,
    pub value: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ChainParameter {
    #[serde(rename = "chainParameter")]
    pub chain_parameter: Vec<ParameterValue>,
}

impl ChainParameter {
    pub fn get_value(&self, key: &str) -> Option<i64> {
        for item in self.chain_parameter.iter() {
            if item.key == key {
                return item.value;
            }
        }
        None
    }

    // unit is sun
    pub fn get_transaction_fee(&self) -> i64 {
        self.get_value("getTransactionFee").unwrap_or(0)
    }

    // unit is sun
    pub fn get_energy_fee(&self) -> i64 {
        self.get_value("getEnergyFee").unwrap_or(0)
    }

    pub fn get_create_account_fee(&self) -> i64 {
        self.get_value("getCreateNewAccountFeeInSystemContract")
            .unwrap_or(0)
            + self.get_value("getCreateAccountFee").unwrap_or(0)
    }

    // 创建账号需要的费用 unit sun
    pub fn get_create_account(&self) -> i64 {
        self.get_value("getCreateNewAccountFeeInSystemContract")
            .unwrap_or(0)
    }
    // 创建账号交易需要的费用
    pub fn get_create_account_transfer_fee(&self) -> i64 {
        self.get_value("getCreateAccountFee").unwrap_or(0)
    }

    pub fn update_account_fee(&self) -> i64 {
        self.get_value("getUpdateAccountPermissionFee")
            .unwrap_or(100_000_000)
    }

    // multisig sign fee ,the unit is sun
    pub fn get_multi_sign_fee(&self) -> i64 {
        self.get_value("getMultiSignFee")
            .unwrap_or(TRX_TO_SUN as i64)
    }
}
