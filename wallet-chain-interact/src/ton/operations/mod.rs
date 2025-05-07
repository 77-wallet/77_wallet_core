use super::errors::TonError;
use tonlib_core::{message::InternalMessage, TonAddress};
pub mod transfer;

pub trait BuildInternalMsg {
    fn build(&self, now_time: u32, bounce: bool) -> Result<InternalMessage, TonError>;

    fn get_src(&self) -> TonAddress;
}
