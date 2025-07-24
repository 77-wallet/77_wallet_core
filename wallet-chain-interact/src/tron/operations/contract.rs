use super::{RawTransactionParams, TronTransactionResponse, transfer::ContractTransferResp};
use crate::{
    abi_encode_address,
    tron::{Provider, params::ResourceConsumer},
};
use alloy::{primitives::U256, sol_types::SolValue as _};
use wallet_utils::hex_func;

// 包装调用合约请求
pub struct WarpContract {
    pub params: TriggerContractParameter,
}

impl WarpContract {
    pub fn new<P>(params: P) -> Result<Self, crate::errors::Error>
    where
        P: TryInto<TriggerContractParameter, Error = crate::Error>,
    {
        let params: Result<TriggerContractParameter, crate::Error> = params.try_into();

        Ok(Self { params: params? })
    }

    // only constant smart contract used to get contract information or estimate energy
    pub async fn trigger_constant_contract(
        &self,
        provider: &Provider,
    ) -> crate::Result<ConstantContract<ContractTransferResp>> {
        let result = provider
            .do_contract_request::<_, _>("wallet/triggerconstantcontract", Some(&self.params))
            .await?;
        Ok(result)
    }

    // build contract transaction
    pub async fn trigger_smart_contract(
        &mut self,
        provider: &Provider,
        consumer: &ResourceConsumer,
    ) -> crate::Result<RawTransactionParams> {
        let fee_limit = consumer.fee_limit();
        let fee_limit = Some(fee_limit + (fee_limit * 20 / 100));

        self.params.fee_limit = fee_limit;

        let result = provider
            .do_contract_request::<_, TriggerContractResult<ContractTransferResp>>(
                "wallet/triggersmartcontract",
                Some(&self.params),
            )
            .await?;

        Ok(RawTransactionParams::from(result.transaction))
    }
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Permission_id")]
    pub permission_id: Option<i64>,
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
            permission_id: None,
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
    pub message: Option<String>,
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

    pub fn parse_num(&self, num: &str) -> crate::Result<U256> {
        let bytes = wallet_utils::hex_func::hex_decode(num)?;
        U256::abi_decode(&bytes, false).map_err(|e| crate::Error::AbiParseError(e.to_string()))
    }

    pub fn is_success(&self) -> Result<(), crate::Error> {
        if let Some(msg) = self.result.message.as_ref() {
            let error_msg = hex_func::hex_to_utf8(msg)?;
            let e =
                crate::ContractValidationError::Other(format!("contract exec error:{}", error_msg));
            return Err(crate::Error::ContractValidationError(e));
        }

        Ok(())
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
