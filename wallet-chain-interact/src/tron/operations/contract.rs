use super::TronTransactionResponse;
use crate::abi_encode_address;
use alloy::{primitives::U256, sol_types::SolValue as _};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct TriggerContractParameter {
    pub contract_address: String,
    pub owner_address: String,
    pub function_selector: String,
    pub parameter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_limit: Option<i64>,
    pub call_value: Option<u64>,
    pub call_token_value: Option<u64>,
    pub token_id: Option<u64>,
}

impl TriggerContractParameter {
    pub fn new(
        contract_address: &str,
        owner_address: &str,
        function_selector: &str,
        parameter: String,
    ) -> Self {
        Self {
            contract_address: contract_address.to_owned(),
            owner_address: owner_address.to_owned(),
            function_selector: function_selector.to_owned(),
            parameter,
            fee_limit: None,
            call_value: Some(0),
            call_token_value: Some(0),
            token_id: Some(0),
        }
    }

    pub fn with_fee_limit(mut self, fee_limit: i64) -> Self {
        self.fee_limit = Some(fee_limit);
        self
    }

    pub fn token_balance_trigger(token: &str, addr: &str) -> crate::Result<Self> {
        let token_addr = wallet_utils::address::bs58_addr_to_hex(token)?;
        let owner = wallet_utils::address::bs58_addr_to_hex(addr)?;

        let function = "balanceOf(address)";
        let parameter = abi_encode_address(&owner);

        Ok(Self::new(&token_addr, &owner, function, parameter))
    }

    pub fn decimal_trigger(token: &str) -> crate::Result<Self> {
        let token_addr = wallet_utils::address::bs58_addr_to_hex(token)?;
        let owner = "ccc41d681485ead2f14afbd3d7df47ccea0bb0128ef54";
        let function = "decimals()";

        Ok(Self::new(&token_addr, owner, function, "".to_string()))
    }

    pub fn symbol_trigger(token: &str) -> crate::Result<Self> {
        let token_addr = wallet_utils::address::bs58_addr_to_hex(token)?;
        let owner = "ccc41d681485ead2f14afbd3d7df47ccea0bb0128ef54";
        let function = "symbol()";

        Ok(Self::new(&token_addr, owner, function, "".to_string()))
    }

    pub fn name_trigger(token: &str) -> crate::Result<Self> {
        let token_addr = wallet_utils::address::bs58_addr_to_hex(token)?;
        let owner = "ccc41d681485ead2f14afbd3d7df47ccea0bb0128ef54";
        let function = "name()";

        Ok(Self::new(&token_addr, owner, function, "".to_string()))
    }

    pub fn black_address(token: &str, owner: &str) -> crate::Result<Self> {
        let token_addr = wallet_utils::address::bs58_addr_to_hex(token)?;
        let owner = wallet_utils::address::bs58_addr_to_hex(owner)?;

        let function = "isBlackListed(address)";
        let parameter = abi_encode_address(&owner);

        Ok(Self::new(&token_addr, &owner, function, parameter))
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct TriggerContractResult<T> {
    pub result: TriggerResult,
    pub transaction: TronTransactionResponse<T>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct TriggerResult {
    pub result: bool,
}

// 类似eth_call
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ConstantContract<T> {
    pub result: TriggerResult,
    pub energy_used: u64,
    pub constant_result: Vec<String>,
    pub transaction: TronTransactionResponse<T>,
    #[serde(flatten)]
    ext: Option<serde_json::Value>,
}

impl<T> ConstantContract<T> {
    /// parse number from abi code
    pub fn parse_u256(&self) -> crate::Result<U256> {
        let bytes = wallet_utils::hex_func::hex_decode(&self.constant_result[0])?;
        U256::abi_decode(&bytes, false).map_err(|e| crate::Error::AbiParseError(e.to_string()))
    }

    /// parse string from abi code
    pub fn parse_string(&self) -> crate::Result<String> {
        let bytes = wallet_utils::hex_func::hex_decode(&self.constant_result[0])?;
        String::from_utf8(bytes).map_err(|e| crate::Error::Other(e.to_string()))
    }

    /// parse bool from abi code
    pub fn parse_bool(&self) -> crate::Result<bool> {
        if self.constant_result[0].len() != 64 {
            return Err(crate::Error::Other(
                "Invalid ABI-encoded hex string length".to_string(),
            ));
        }

        match self.constant_result[0].as_str() {
            "0000000000000000000000000000000000000000000000000000000000000001" => Ok(true),
            "0000000000000000000000000000000000000000000000000000000000000000" => Ok(false),
            _ => Err(crate::Error::Other(
                "Invalid ABI encoding for boolean".to_string(),
            )),
        }
    }
}
