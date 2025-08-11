// 封装erc20相关的操作
use crate::{
    eth::protocol::contract::{allowanceCall, approveCall, depositCall, withdrawCall},
    types,
};
use alloy::{
    network::TransactionBuilder as _,
    primitives::{self, U256},
    rpc::types::TransactionRequest,
    sol_types::SolCall as _,
};
use wallet_utils::address;

pub struct Approve {
    pub spender: primitives::Address,
    pub from: primitives::Address,
    pub amount: primitives::U256,
    pub contract: primitives::Address,
}

impl Approve {
    pub fn new(
        from: &str,
        spender: &str,
        amount: alloy::primitives::U256,
        contract: &str,
    ) -> crate::Result<Self> {
        let spender = address::parse_eth_address(spender)?;
        let contract = address::parse_eth_address(contract)?;

        Ok(Self {
            spender,
            from: address::parse_eth_address(from)?,
            amount,
            contract,
        })
    }
}

impl types::Transaction<TransactionRequest> for Approve {
    fn build_transaction(&self) -> Result<TransactionRequest, crate::Error> {
        let call = approveCall {
            spender: self.spender,
            amount: self.amount,
        };

        Ok(TransactionRequest::default()
            .from(self.from)
            .to(self.contract)
            .value(primitives::U256::ZERO)
            .with_input(call.abi_encode()))
    }
}

pub struct Deposit {
    pub from: primitives::Address,
    pub amount: primitives::U256,
    pub contract: primitives::Address,
}

impl Deposit {
    pub fn new(from: &str, contract: &str, amount: alloy::primitives::U256) -> crate::Result<Self> {
        let from = address::parse_eth_address(from)?;
        let contract = address::parse_eth_address(contract)?;

        Ok(Self {
            from,
            amount,
            contract,
        })
    }
}

impl types::Transaction<TransactionRequest> for Deposit {
    fn build_transaction(&self) -> Result<TransactionRequest, crate::Error> {
        let call = depositCall {};

        Ok(TransactionRequest::default()
            .from(self.from)
            .to(self.contract)
            .value(self.amount)
            .with_input(call.abi_encode()))
    }
}

pub struct Withdraw {
    pub from: primitives::Address,
    pub amount: primitives::U256,
    pub contract: primitives::Address,
}

impl Withdraw {
    pub fn new(from: &str, contract: &str, amount: alloy::primitives::U256) -> crate::Result<Self> {
        let from = address::parse_eth_address(from)?;
        let contract = address::parse_eth_address(contract)?;

        Ok(Self {
            from,
            amount,
            contract,
        })
    }
}

impl types::Transaction<TransactionRequest> for Withdraw {
    fn build_transaction(&self) -> Result<TransactionRequest, crate::Error> {
        let call = withdrawCall {
            amount: self.amount,
        };

        Ok(TransactionRequest::default()
            .from(self.from)
            .to(self.contract)
            .with_input(call.abi_encode()))
    }
}

pub struct Allowance {
    pub from: primitives::Address,
    pub spender: primitives::Address,
    pub contract: primitives::Address,
}

impl Allowance {
    pub fn new(from: &str, contract: &str, spender: &str) -> crate::Result<Self> {
        let from = address::parse_eth_address(from)?;
        let spender = address::parse_eth_address(spender)?;
        let contract = address::parse_eth_address(contract)?;

        Ok(Self {
            from,
            spender,
            contract,
        })
    }
}

impl types::Transaction<TransactionRequest> for Allowance {
    fn build_transaction(&self) -> Result<TransactionRequest, crate::Error> {
        let call = allowanceCall {
            owner: self.from,
            spender: self.spender,
        };

        Ok(TransactionRequest::default()
            .from(self.from)
            .to(self.contract)
            .value(U256::ZERO)
            .with_input(call.abi_encode()))
    }
}
