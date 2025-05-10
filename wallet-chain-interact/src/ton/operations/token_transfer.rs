use crate::ton::{
    address::parse_addr_from_bs64_url, errors::TonError, protocol::jettons::JettonWalletAddress,
    provider::Provider,
};
use alloy::primitives::U256;
use async_trait::async_trait;
use num_bigint::BigUint;
use std::{str::FromStr, sync::Arc};
use tonlib_core::{
    cell::{Cell, EitherCellLayout},
    message::{
        CommonMsgInfo, InternalMessage, JettonTransferMessage, TonMessage as _, TransferMessage,
    },
    TonAddress,
};

use super::BuildInternalMsg;

pub struct TokenTransferOpt {
    pub from: TonAddress,
    pub to: TonAddress,
    pub token: String,
    pub value: BigUint,
}

impl TokenTransferOpt {
    pub fn new(from: &str, to: &str, token: &str, value: U256) -> crate::Result<Self> {
        let value = value.to_string();

        let value = BigUint::from_str(&value)
            .map_err(|e| crate::Error::ParseError(crate::ParseErr::ValueErr(e.to_string())))?;

        Ok(Self {
            from: parse_addr_from_bs64_url(from)?,
            to: parse_addr_from_bs64_url(to)?,
            token: token.to_string(),
            value,
        })
    }

    fn transfer_body(&self) -> Result<Cell, TonError> {
        // TODO : forward to amount
        let jetton_transfer = JettonTransferMessage {
            query_id: wallet_utils::time::now().timestamp() as u64,
            amount: self.value.clone(),
            destination: self.to.clone(),
            response_destination: self.from.clone(),
            custom_payload: None,
            forward_ton_amount: BigUint::from(1u32),
            forward_payload: Arc::new(Cell::default()),
            forward_payload_layout: EitherCellLayout::Native,
        }
        .build()?;

        Ok(jetton_transfer)
    }
}

#[async_trait]
impl BuildInternalMsg for TokenTransferOpt {
    async fn build(
        &self,
        now_time: u32,
        bounce: bool,
        provider: &Provider,
    ) -> Result<TransferMessage, crate::ton::errors::TonError> {
        // 代币转账参数
        let jetton_transfer = self.transfer_body()?;

        let src_jetton_address =
            JettonWalletAddress::wallet_address(&self.token, &self.from.to_base64_url(), provider)
                .await?;

        let ton_amount = BigUint::from(10000000u64);
        let internal = InternalMessage {
            ihr_disabled: true,
            bounce,
            bounced: false,
            src: self.from.clone(),
            dest: src_jetton_address,
            value: ton_amount,
            ihr_fee: 0u32.into(),
            fwd_fee: 0u32.into(),
            created_lt: 0,
            created_at: now_time,
        };

        let common_msg_info = CommonMsgInfo::InternalMessage(internal);
        let transfer = TransferMessage::new(common_msg_info)
            .with_data(jetton_transfer.into())
            .to_owned();

        Ok(transfer)
    }

    fn get_src(&self) -> TonAddress {
        self.from.clone()
    }
}
