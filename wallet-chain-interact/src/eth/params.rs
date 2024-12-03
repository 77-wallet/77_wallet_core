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

// use crate::params::TransactionParams;
// use crate::types::{ExecTransactionParams, FeeCalculator};
// use crate::{convert_eth_address, TransactionFeeSetting};

// pub(super) struct EthTransactionParams {
//     pub from: Address,
//     pub to: Address,
//     pub token_address: Option<Address>,
//     pub value: U256,
//     pub private_key: Option<String>,
//     pub fee_setting: TransactionFeeSetting,
// }

// impl EthTransactionParams {
//     pub fn get_private_key(&self) -> String {
//         match self.private_key.clone() {
//             Some(key) => {
//                 if !key.starts_with("0x") {
//                     format!("0x{}", key)
//                 } else {
//                     key
//                 }
//             }
//             None => "".to_string(),
//         }
//     }

//     pub fn get_token_address(&self) -> Address {
//         match self.token_address {
//             Some(addr) => addr,
//             None => panic!("token address is none"),
//         }
//     }
// }
// impl TryFrom<TransactionParams> for EthTransactionParams {
//     type Error = crate::Error;
//     fn try_from(value: TransactionParams) -> Result<Self, Self::Error> {
//         let token = value
//             .token_address
//             .map(|addr| convert_eth_address(&addr).unwrap());

//         // TODO error handel
//         let amount = wallet_utils::unit::convert_to_u256(&value.value, value.decimals)
//             .map_err(|e| crate::Error::Other(e.to_string()))
//             .unwrap();

//         Ok(Self {
//             from: convert_eth_address(&value.from)?,
//             to: convert_eth_address(&value.to)?,
//             token_address: token,
//             value: amount,
//             private_key: value.private_key,
//             fee_setting: value.fee_setting,
//         })
//     }
// }

// impl TryFrom<ExecTransactionParams> for EthTransactionParams {
//     type Error = crate::Error;
//     fn try_from(value: ExecTransactionParams) -> Result<Self, Self::Error> {
//         let executor = value.executor.unwrap();

//         let fee = TransactionFeeSetting::default();

//         Ok(Self {
//             from: convert_eth_address(&executor.executor)?,
//             to: convert_eth_address(&executor.destination)?,
//             token_address: None,
//             value: U256::ZERO,
//             private_key: Some(executor.private_key),
//             fee_setting: fee,
//         })
//     }
// }
