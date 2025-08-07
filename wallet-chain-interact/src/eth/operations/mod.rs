use alloy::primitives;
pub mod multisig;
pub use multisig::*;
pub mod transfer;
pub use transfer::*;
pub mod erc;
use wallet_utils::address;

pub struct EthereumBaseTransaction {
    pub from: primitives::Address,
    pub to: primitives::Address,
    pub value: primitives::U256,
}

impl EthereumBaseTransaction {
    pub fn new(from: &str, to: &str, value: primitives::U256) -> crate::Result<Self> {
        let from = address::parse_eth_address(from)?;
        let to = address::parse_eth_address(to)?;
        Ok(Self { from, to, value })
    }
}
