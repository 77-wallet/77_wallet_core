use num_bigint::BigUint;
use std::sync::Arc;
use tonlib_core::{
    cell::CellBuilder,
    message::{CommonMsgInfo, ExternalIncomingMessage, TransferMessage},
    TonAddress,
};

pub struct TransferOpt {
    pub from: String,
    pub to: String,
    pub value: String,
}

impl TransferOpt {
    pub fn new(from: &str, to: &str, value: &str) -> crate::Result<Self> {
        Ok(Self {
            from: from.to_string(),
            to: to.to_string(),
            value: value.to_string(),
        })
    }

    pub fn to_message(&self) -> crate::Result<TransferMessage> {
        let src = TonAddress::from_base64_url(&self.from).unwrap();
        let dest = TonAddress::from_base64_url(&self.to).unwrap();
        let value = BigUint::from(10_000_000u32);

        let external_msg_info = ExternalIncomingMessage {
            src: src.clone(),
            dest: dest.clone(),
            import_fee: value.clone(),
        };

        let mut msg_builder = CellBuilder::new();

        msg_builder.store_u32(6, 0b10).unwrap();
        msg_builder.store_address(&dest).unwrap();
        msg_builder.store_coins(&value).unwrap();
        msg_builder.store_u8(8, 0).unwrap(); // no flags
        msg_builder.store_u32(32, 0).unwrap(); // empty payload
        let internal_msg = Arc::new(msg_builder.build().unwrap());

        let common_msg_info = CommonMsgInfo::ExternalIncomingMessage(external_msg_info);

        let mut msg = TransferMessage::new(common_msg_info);
        msg.with_data(internal_msg.into());

        Ok(msg)
    }
}
