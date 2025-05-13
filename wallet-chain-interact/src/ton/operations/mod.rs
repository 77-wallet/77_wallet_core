use std::sync::Arc;

use super::{errors::TonError, provider::Provider};
use async_trait::async_trait;
use tonlib_core::{
    cell::Cell,
    message::{TonMessage as _, TransferMessage},
    wallet::{
        version_helper::VersionHelper, versioned::DEFAULT_WALLET_ID, wallet_version::WalletVersion,
    },
    TonAddress,
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
    ) -> crate::Result<Cell> {
        let version = match address_type {
            TonAddressType::V4R2 => WalletVersion::V4R2,
            TonAddressType::V5R1 => WalletVersion::V5R1,
        };
        let trans = trans.build().map_err(TonError::TonMsg)?;
        let msgs_refs = vec![Arc::new(trans)];

        let ext_msg = VersionHelper::build_ext_msg(
            version,
            now_time + 60,
            seqno,
            DEFAULT_WALLET_ID,
            msgs_refs,
        )
        .map_err(TonError::CellBuild)?;
        Ok(ext_msg)
    }
}
