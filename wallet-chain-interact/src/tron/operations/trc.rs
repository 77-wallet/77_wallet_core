use super::contract::TriggerContractParameter;
use crate::{
    abi_encode_address, abi_encode_u256,
    tron::consts::{self, TRX_TO_SUN},
};

pub struct Deposit {
    pub owner_address: String,
    pub contract: String,
    // unit is sun
    pub value: u64,
}

impl Deposit {
    pub fn new(owner_address: &str, contract: &str, value: f64) -> Self {
        Deposit {
            owner_address: owner_address.to_string(),
            contract: contract.to_string(),
            value: (value * TRX_TO_SUN as f64) as u64,
        }
    }
}

impl TryFrom<Deposit> for TriggerContractParameter {
    type Error = crate::errors::Error;

    fn try_from(value: Deposit) -> Result<Self, Self::Error> {
        let contract_address = wallet_utils::address::bs58_addr_to_hex(&value.contract)?;
        let owner_address = wallet_utils::address::bs58_addr_to_hex(&value.owner_address)?;

        let function_selector = "deposit()";

        let mut res = TriggerContractParameter::new(
            &contract_address,
            &owner_address,
            function_selector,
            "".to_string(),
        );
        res.call_value = Some(value.value);

        Ok(res)
    }
}

pub struct Approve {
    pub owner_address: String,
    pub to_address: String,
    pub contract: String,
    // unit is sun
    pub value: u64,
}

impl Approve {
    pub fn new(owner_address: &str, to_address: &str, contract: &str, value: f64) -> Self {
        Approve {
            owner_address: owner_address.to_string(),
            to_address: to_address.to_string(),
            contract: contract.to_string(),
            value: (value * TRX_TO_SUN as f64) as u64,
        }
    }
}

impl TryFrom<Approve> for TriggerContractParameter {
    type Error = crate::errors::Error;

    fn try_from(value: Approve) -> Result<Self, Self::Error> {
        let contract_address: String = wallet_utils::address::bs58_addr_to_hex(&value.contract)?;
        let owner_address = wallet_utils::address::bs58_addr_to_hex(&value.owner_address)?;

        let amount =
            wallet_utils::unit::convert_to_u256(&value.value.to_string(), consts::TRX_DECIMALS)?;
        let function_selector = "approve(address,uint256)";

        let to = wallet_utils::address::bs58_addr_to_hex(&value.to_address)?;
        let parameter = format!("{}{}", abi_encode_address(&to), abi_encode_u256(amount));

        let res = TriggerContractParameter::new(
            &contract_address,
            &owner_address,
            function_selector,
            parameter,
        );

        Ok(res)
    }
}
