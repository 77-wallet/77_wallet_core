use std::sync::Arc;
use tonlib_core::{
    cell::{BagOfCells, CellBuilder},
    TonAddress,
};

use crate::ton::provider::Provider;

use super::common::RunGetMethodParams;

#[derive(Debug, serde::Deserialize)]
pub struct TokenDataResp {
    pub balance: u64,
    pub owner: String,
    pub jetton: String,
    pub jetton_wallet_code: String,
    pub contract_type: String,
}

pub struct JettonWalletAddress;

impl JettonWalletAddress {
    // local calculate jetton wallet address
    pub fn jetton_wallet_address(
        code: &str,
        jetton_master: &str,
        owner: &str,
    ) -> crate::Result<TonAddress> {
        let jetton_master = TonAddress::from_base64_url(jetton_master).unwrap();
        let owner = TonAddress::from_base64_url(owner).unwrap();

        let data = CellBuilder::new()
            .store_address(&jetton_master)
            .unwrap()
            .store_address(&owner)
            .unwrap()
            .build()
            .unwrap();

        let code = BagOfCells::parse_base64(code)
            .unwrap()
            .single_root()
            .unwrap();

        let c = TonAddress::derive(0, code, Arc::new(data)).unwrap();

        Ok(c)
    }

    // get jetton wallet address by contract
    pub async fn wallet_address(
        jetton_master: &str,
        owner: &str,
        provider: &Provider,
    ) -> crate::Result<TonAddress> {
        let address = TonAddress::from_base64_url(owner).unwrap();

        let mut builder = CellBuilder::new();
        builder.store_address(&address).unwrap();
        let cell = builder.build().unwrap();
        let boc = BagOfCells::from_root(cell).serialize(false).unwrap();
        let address = wallet_utils::bytes_to_base64(&boc);

        tracing::warn!("add {}", address);

        let slice_param = vec!["tvm.Slice".to_string(), address];
        let stack = vec![slice_param];

        let params = RunGetMethodParams::new(jetton_master, "get_wallet_address", stack);

        let response = provider.run_get_method(params).await.unwrap();

        match &response.stack[0] {
            super::common::StackItem::Slice(_, r) => {
                let bag = BagOfCells::parse_base64(&r.bytes).unwrap();

                let cell = bag.single_root().unwrap();

                Ok(cell.parser().load_address().unwrap())
            }
            _ => panic!("runGetMethod response error"),
        }
    }
}

#[test]
fn test_add() {
    let bag = BagOfCells::parse_base64(
        "te6cckEBAQEAJAAAQ4ASFYBStVb6FFw67Qm7QBnRb0vG2Rlrwi+kZgLPfhbTmFCOM0d3",
    )
    .unwrap();
    let cell = bag.single_root().unwrap();

    let data = cell.parser().load_address().unwrap();
    println!("address {}", data.to_base64_url());
}
