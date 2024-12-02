use super::protocol::{
    protobuf::transaction::Raw,
    transaction::{CreateTransactionResp, SendRawTransactionParams},
};
use protobuf::Message;
use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use wallet_utils::sha256;

pub struct TransactionBuilder;

impl TransactionBuilder {
    pub fn build_raw_transaction_v2<T: Serialize>(
        mut resp: CreateTransactionResp<T>,
        modify_expiration: bool,
    ) -> crate::Result<SendRawTransactionParams> {
        let mut tx_id = resp.tx_id;
        let mut raw_data_hex = resp.raw_data_hex;

        if modify_expiration {
            let system_time = UNIX_EPOCH + Duration::from_millis(resp.raw_data.expiration);

            let new_time = TransactionBuilder::get_new_time(system_time);
            resp.raw_data.expiration = new_time as u64;

            let mut raw = Raw::parse_from_bytes(&hex::decode(raw_data_hex).unwrap()).unwrap();
            raw.expiration = new_time;

            let bytes = raw.write_to_bytes().unwrap();
            tx_id = hex::encode(sha256(&bytes));
            raw_data_hex = hex::encode(bytes);
        }

        let raw_data_str = serde_json::to_string(&resp.raw_data).unwrap();

        let payload = SendRawTransactionParams {
            tx_id,
            raw_data: raw_data_str,
            raw_data_hex,
            signature: vec![],
        };

        Ok(payload)
    }

    fn get_new_time(system_time: SystemTime) -> i64 {
        let one_day = Duration::from_millis(86400 * 1000);
        let new_system_time = system_time + one_day;

        new_system_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as i64
    }
}
