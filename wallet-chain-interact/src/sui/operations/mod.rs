pub mod transfer;
use sui_sdk::types::base_types::SuiAddress;
use sui_types::base_types::ObjectRef;
pub use transfer::*;
use wallet_utils::address;

pub struct SuiBaseTransaction {
    pub from: SuiAddress,
    pub to: SuiAddress,
    pub amount: u64,
    /// 要转移的 Coin 对象引用 (ObjectRef)
    // pub input_coins: Vec<ObjectRef>,
    /// 用于支付 gas 的对象引用 (ObjectRef)
    pub transfer_coins: Vec<ObjectRef>,
    pub gas_coins: Vec<ObjectRef>,
    // pub gas_budget: u64,
    // pub gas_price: u64,
}

impl SuiBaseTransaction {
    pub fn new(
        from: &str,
        to: &str,
        amount: u64,
        // input_coins: Vec<ObjectRef>,
        transfer_coins: Vec<ObjectRef>,
        gas_coins: Vec<ObjectRef>,
        // gas_budget: u64,
        // gas_price: u64,
    ) -> crate::Result<Self> {
        let from = address::parse_sui_address(from)?;
        // let recipients = recipients
        //     .into_iter()
        //     .map(address::parse_sui_address)
        //     .collect::<Result<Vec<_>, _>>()?;
        let to = address::parse_sui_address(to)?;
        Ok(Self {
            from,
            to,
            amount,
            // input_coins,
            transfer_coins,
            gas_coins, // gas_budget,
                       // gas_price,
        })
    }
}
