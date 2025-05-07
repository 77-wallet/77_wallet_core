pub mod transfer;
use sui_sdk::types::base_types::SuiAddress;
pub use transfer::*;
use wallet_utils::address;

pub struct SuiBaseTransaction {
    pub from: SuiAddress,
    pub to: SuiAddress,
    pub value: u128,
}

impl SuiBaseTransaction {
    pub fn new(from: &str, to: &str, value: u128) -> crate::Result<Self> {
        let from = address::parse_sui_address(from)?;
        let to = address::parse_sui_address(to)?;
        Ok(Self { from, to, value })
    }
}
