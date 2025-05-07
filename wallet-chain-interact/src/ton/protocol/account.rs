use super::{block::BlockIdExt, common::RunGetMethodParams, transaction::TransactionId};
use crate::ton::provider::Provider;

#[derive(Debug, serde::Deserialize)]
pub struct AddressInformation {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub balance: String,
    pub code: String,
    pub data: String,
    pub last_transaction_id: TransactionId,
    pub block_id: BlockIdExt,
    pub frozen_hash: String,
    pub sync_utime: u64,
    #[serde(rename = "@extra")]
    pub extra: String,
    pub state: String,
}
impl AddressInformation {
    pub async fn seqno(address: &str, provider: &Provider) -> crate::Result<u32> {
        let params = RunGetMethodParams::<()>::new(address, "seqno", vec![]);
        let result = provider.run_get_method(params).await?;

        match &result.stack[0] {
            super::common::StackItem::Num(_, r) => {
                let value = u32::from_str_radix(r.trim_start_matches("0x"), 16)
                    .expect("get seqno invalid hex");
                Ok(value)
            }
            _ => panic!("error"),
        }
    }
}
