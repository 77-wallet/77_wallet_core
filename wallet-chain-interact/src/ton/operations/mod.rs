use super::{errors::TonError, provider::Provider};
use async_trait::async_trait;
use tonlib_core::{message::TransferMessage, TonAddress};
pub mod token_transfer;
pub mod transfer;

#[async_trait]
pub trait BuildInternalMsg {
    async fn build(
        &self,
        now_time: u32,
        bounce: bool,
        provider: &Provider,
    ) -> Result<TransferMessage, TonError>;

    fn get_src(&self) -> TonAddress;
}
