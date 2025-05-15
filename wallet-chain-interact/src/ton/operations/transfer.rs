use super::BuildInternalMsg;
use crate::ton::{
    address::parse_addr_from_bs64_url, consts::TON_DECIMAL, protocol::account::AddressInformation,
    provider::Provider,
};
use async_trait::async_trait;
use num_bigint::BigUint;
use tonlib_core::{
    cell::Cell,
    message::{CommonMsgInfo, InternalMessage, TransferMessage},
    TonAddress,
};
use wallet_types::chain::address::r#type::TonAddressType;

pub struct TransferOpt {
    pub from: TonAddress,
    pub to: TonAddress,
    pub value: u64,
    pub spend_all: bool,
}

impl TransferOpt {
    pub fn new(from: &str, to: &str, value: &str, spend_all: bool) -> crate::Result<Self> {
        let value = wallet_utils::unit::convert_to_u256(value, TON_DECIMAL)?.to::<u64>();

        Ok(Self {
            from: parse_addr_from_bs64_url(from)?,
            to: parse_addr_from_bs64_url(to)?,
            value,
            spend_all,
        })
    }

    pub fn internal_msg(&self, bounce: bool, now_time: u32) -> InternalMessage {
        InternalMessage {
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
        }
    }
}

#[async_trait]
impl BuildInternalMsg for TransferOpt {
    async fn build_trans(
        &self,
        address_type: TonAddressType,
        provider: &Provider,
    ) -> crate::Result<Cell> {
        let now_time = wallet_utils::time::now().timestamp() as u32;
        let bounce = false;

        let internal_msg = self.internal_msg(bounce, now_time);
        let common_msg_info = CommonMsgInfo::InternalMessage(internal_msg);
        let trans = TransferMessage::new(common_msg_info);

        let seqno = AddressInformation::seqno(self.from.clone(), provider).await?;

        self.build_ext_msg(trans, address_type, now_time, seqno, self.spend_all)
    }

    fn get_src(&self) -> TonAddress {
        self.from.clone()
    }
}
