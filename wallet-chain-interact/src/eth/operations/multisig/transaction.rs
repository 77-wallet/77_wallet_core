use std::str::FromStr as _;

use crate::eth::protocol::contract;
use crate::types;
use alloy::{
    hex::FromHex as _, network::TransactionBuilder, primitives, rpc::types::TransactionRequest,
    sol_types::SolCall,
};
use wallet_utils::{address, hex_func, sign};

pub struct MultisigTransferOpt {
    // 多签账户地址
    pub multisig_account: primitives::Address,
    // 给谁转账
    pub to: primitives::Address,
    pub value: primitives::U256,
    pub nonce: primitives::U256,
    pub token: Option<primitives::Address>,
    pub exec_params: Option<ExecParams>,
}
pub struct ExecParams {
    pub from: primitives::Address,
    pub input_data: String,
    pub signatures: String,
}

impl MultisigTransferOpt {
    pub fn new(
        multisig_account: &str,
        to: &str,
        value: primitives::U256,
    ) -> Result<Self, crate::Error> {
        let multisig_account = address::parse_eth_address(multisig_account)?;
        let to = address::parse_eth_address(to)?;

        Ok(Self {
            multisig_account,
            to,
            value,
            nonce: primitives::U256::ZERO,
            token: None,
            exec_params: None,
        })
    }

    pub fn with_token(mut self, token: Option<String>) -> Result<Self, crate::Error> {
        if let Some(token) = token {
            self.token = Some(address::parse_eth_address(&token)?);
        }
        Ok(self)
    }

    pub fn exec_params(
        mut self,
        from: &str,
        raw_data: String,
        signatures: String,
    ) -> Result<Self, crate::Error> {
        let address = address::parse_eth_address(from)?;
        let payload = MultisigPayloadOpt::from_str(&raw_data)?;

        self.exec_params = Some(ExecParams {
            from: address,
            input_data: payload.input_data,
            signatures,
        });
        Ok(self)
    }

    pub fn nonce_tx(&self) -> Result<TransactionRequest, crate::Error> {
        let call = contract::nonceCall {};

        Ok(TransactionRequest::default()
            .to(self.multisig_account)
            .with_input(call.abi_encode()))
    }
}

impl MultisigTransferOpt {
    pub fn build_request(&self) -> Result<TransactionRequest, crate::Error> {
        let default_value = primitives::U256::ZERO;
        let default_address = primitives::Address::default();

        let tx_data = if let Some(token) = self.token {
            let data = contract::transferCall {
                from: self.to,
                amount: self.value,
            };

            contract::getTransactionHashCall {
                to: token,
                value: primitives::U256::ZERO,
                data: data.abi_encode().into(),
                operation: 0,
                safeTxGas: default_value,
                baseGas: default_value,
                gasPrice: default_value,
                gasToken: default_address,
                refundReceiver: default_address,
                _nonce: self.nonce,
            }
        } else {
            contract::getTransactionHashCall {
                to: self.to,
                value: self.value,
                data: primitives::Bytes::default(),
                operation: 0,
                safeTxGas: default_value,
                baseGas: default_value,
                gasPrice: default_value,
                gasToken: default_address,
                refundReceiver: default_address,
                _nonce: self.nonce,
            }
        };

        Ok(TransactionRequest::default()
            .with_to(self.multisig_account)
            .with_input(tx_data.abi_encode()))
    }

    pub fn from_input_data(&self, exec: &ExecParams) -> Result<TransactionRequest, crate::Error> {
        let input_str = if exec.input_data.starts_with("0x") {
            exec.input_data[2..].to_string()
        } else {
            exec.input_data.to_string()
        };
        let bytes = hex::decode(input_str.clone()).unwrap();

        let hash_call = contract::getTransactionHashCall::abi_decode(&bytes, false)
            .map_err(|e| crate::Error::HexError(e.to_string()))?;
        let signatures = primitives::Bytes::from_hex(&exec.signatures).unwrap();

        let tx_data = contract::execTransactionCall {
            to: hash_call.to,
            data: hash_call.data,
            value: hash_call.value,
            operation: hash_call.operation,
            safeTxGas: hash_call.safeTxGas,
            baseGas: hash_call.baseGas,
            gasPrice: hash_call.gasPrice,
            gasToken: hash_call.gasToken,
            refundReceiver: hash_call.refundReceiver,
            signatures,
        };

        Ok(TransactionRequest::default()
            .with_from(exec.from)
            .with_to(self.multisig_account)
            .with_input(tx_data.abi_encode()))
    }
}

impl types::Transaction<TransactionRequest> for MultisigTransferOpt {
    fn build_transaction(&self) -> Result<TransactionRequest, crate::Error> {
        if let Some(exec) = self.exec_params.as_ref() {
            self.from_input_data(exec)
        } else {
            self.build_request()
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MultisigPayloadOpt {
    pub input_data: String,
    pub sign_message: String,
}

impl std::str::FromStr for MultisigPayloadOpt {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(hex_func::bincode_decode(s)?)
    }
}

impl MultisigPayloadOpt {
    pub fn new(input_data: String, sign_message: String) -> Self {
        Self {
            input_data,
            sign_message,
        }
    }
    pub fn to_string(&self) -> crate::Result<String> {
        Ok(hex_func::bincode_encode(&self)?)
    }

    pub fn sign_message(
        &self,
        key: types::ChainPrivateKey,
    ) -> crate::Result<types::MultisigSignResp> {
        let signature = sign::sign_tron(&self.sign_message, &key, Some(27))?;
        Ok(types::MultisigSignResp::new(signature))
    }
}
