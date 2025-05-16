use super::{errors::TonError, provider::Provider};
use async_trait::async_trait;
use std::sync::Arc;
use tonlib_core::{
    TonAddress,
    cell::{ArcCell, Cell, TonCellError},
    message::{TonMessage as _, TransferMessage},
    tlb_types::traits::TLBObject as _,
    wallet::{
        versioned::{DEFAULT_WALLET_ID, v4::WalletExtMsgBodyV4, v5::WalletExtMsgBodyV5},
        wallet_version::WalletVersion,
    },
};
use wallet_types::chain::address::r#type::TonAddressType;
pub mod token_transfer;
pub mod transfer;

#[async_trait]
pub trait BuildInternalMsg {
    async fn build_trans(
        &self,
        address_type: TonAddressType,
        provider: &Provider,
    ) -> crate::Result<Cell>;

    fn get_src(&self) -> TonAddress;

    fn build_ext_msg(
        &self,
        trans: TransferMessage,
        address_type: TonAddressType,
        now_time: u32,
        seqno: u32,
        spend_all: bool,
    ) -> crate::Result<Cell> {
        let version = address_type.to_version();

        let trans = trans.build().map_err(TonError::TonMsg)?;
        let msgs_refs = vec![Arc::new(trans)];

        let msg_mode = if spend_all { 144 } else { 3 };
        let ext_msg = build_ext_msg(
            version,
            now_time + 60,
            seqno,
            DEFAULT_WALLET_ID,
            msgs_refs,
            msg_mode,
        )
        .map_err(TonError::CellBuild)?;

        Ok(ext_msg)
    }
}

fn build_ext_msg<T: AsRef<[ArcCell]>>(
    version: WalletVersion,
    valid_until: u32,
    msg_seqno: u32,
    wallet_id: i32,
    msgs_refs: T,
    msg_mode: u8,
) -> Result<Cell, TonCellError> {
    let msgs: Vec<ArcCell> = msgs_refs.as_ref().to_vec();

    match version {
        WalletVersion::V4R1 | WalletVersion::V4R2 => WalletExtMsgBodyV4 {
            subwallet_id: wallet_id,
            valid_until,
            msg_seqno,
            opcode: 0,
            msgs_modes: vec![msg_mode; msgs.len()],
            msgs,
        }
        .to_cell(),
        WalletVersion::V5R1 => WalletExtMsgBodyV5 {
            wallet_id,
            valid_until,
            msg_seqno,
            msgs_modes: vec![msg_mode; msgs.len()],
            msgs,
        }
        .to_cell(),
        _ => {
            let err_str = format!("build_ext_msg for {version:?} is unsupported");
            Err(TonCellError::InternalError(err_str))
        }
    }
}
