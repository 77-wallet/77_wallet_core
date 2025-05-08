pub mod transfer;
use sui_sdk::types::base_types::SuiAddress;
use sui_types::base_types::{ObjectID, ObjectRef};
pub use transfer::*;
use wallet_utils::address;

pub struct SuiBaseTransaction {
    pub from: SuiAddress,
    pub to: SuiAddress,
    pub amount: u128,
    /// 要转移的 Coin 对象引用 (ObjectRef)
    pub transfer_object_ref: ObjectRef,
    /// 用于支付 gas 的对象引用 (ObjectRef)
    pub gas_payment_ref: ObjectRef,
    pub gas_budget: u64,
    pub gas_price: u64,
}

impl SuiBaseTransaction {
    pub fn new(
        from: &str,
        to: &str,
        amount: u128,
        transfer_object_ref: ObjectRef,
        gas_payment_ref: ObjectRef,
        gas_budget: u64,
        gas_price: u64,
    ) -> crate::Result<Self> {
        let from = address::parse_sui_address(from)?;
        let to = address::parse_sui_address(to)?;
        Ok(Self {
            from,
            to,
            amount,
            transfer_object_ref,
            gas_payment_ref,
            gas_budget,
            gas_price,
        })
    }
}
