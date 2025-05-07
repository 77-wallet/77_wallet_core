use super::BuildInternalMsg;
use crate::ton::{address::parse_addr_from_bs64_url, consts::TON_DECIMAL};
use num_bigint::BigUint;
use tonlib_core::{message::InternalMessage, TonAddress};

pub struct TransferOpt {
    pub from: TonAddress,
    pub to: TonAddress,
    pub value: u64,
}

impl TransferOpt {
    pub fn new(from: &str, to: &str, value: &str) -> crate::Result<Self> {
        let value = wallet_utils::unit::convert_to_u256(value, TON_DECIMAL)?.to::<u64>();

        Ok(Self {
            from: parse_addr_from_bs64_url(from)?,
            to: parse_addr_from_bs64_url(to)?,
            value,
        })
    }
}

impl BuildInternalMsg for TransferOpt {
    fn build(
        &self,
        now_time: u32,
        bounce: bool,
    ) -> Result<InternalMessage, crate::ton::errors::TonError> {
        let internal = InternalMessage {
            ihr_disabled: true,
            bounce,
            bounced: false,
            src: self.from.clone(),
            dest: self.to.clone(),
            value: BigUint::from(self.value),
            ihr_fee: 0u32.into(),
            fwd_fee: 0u32.into(),
            created_lt: 0,
            created_at: now_time,
        };
        Ok(internal)
    }

    fn get_src(&self) -> TonAddress {
        self.from.clone()
    }
}
