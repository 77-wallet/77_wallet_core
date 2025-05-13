use super::common::RunGetMethodParams;
use crate::ton::{
    address::parse_addr_from_bs64_url, consts::DEFAULT_WORKCHAIN, errors::TonError,
    provider::Provider,
};
use std::sync::Arc;
use tonlib_core::{
    cell::{BagOfCells, CellBuilder},
    TonAddress,
};

#[derive(Debug, serde::Deserialize)]
pub struct JettonMasterResp {
    pub total_supply: serde_json::Value,
    pub mintable: bool,
    pub admin_address: Option<String>,
    pub jetton_content: JettonContent,
    pub jetton_wallet_code: String,
    pub contract_type: String,
}

impl JettonMasterResp {
    pub fn decimal(&self) -> crate::Result<u8> {
        Ok(wallet_utils::unit::str_to_num(
            &self.jetton_content.data.decimals,
        )?)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct JettonMeta {
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct JettonContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub data: JettonData,
}

#[derive(Debug, serde::Deserialize)]
pub struct JettonData {
    pub uri: String,
    pub decimals: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct JettonWalletResp {
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
    ) -> Result<TonAddress, TonError> {
        let jetton_master = parse_addr_from_bs64_url(jetton_master)?;
        let owner = parse_addr_from_bs64_url(owner)?;

        let data = CellBuilder::new()
            .store_address(&jetton_master)?
            .store_address(&owner)?
            .build()?;

        let code = BagOfCells::parse_base64(code)?.single_root()?;

        Ok(TonAddress::derive(DEFAULT_WORKCHAIN, code, Arc::new(data))?)
    }

    // get jetton wallet address by contract
    pub async fn wallet_address(
        jetton_master: &str,
        owner: &str,
        provider: &Provider,
    ) -> Result<TonAddress, TonError> {
        let address = TonAddress::from_base64_url(owner)?;

        let mut builder = CellBuilder::new();
        let cell = builder.store_address(&address)?.build()?;

        let boc = BagOfCells::from_root(cell).serialize(false)?;
        let address = wallet_utils::bytes_to_base64(&boc);

        let slice_param = vec!["tvm.Slice".to_string(), address];

        let params =
            RunGetMethodParams::new(jetton_master, "get_wallet_address", vec![slice_param]);

        let response = provider.run_get_method(params).await?;

        match &response.stack[0] {
            super::common::StackItem::Slice(_, r) => {
                let cell = BagOfCells::parse_base64(&r.bytes)?.single_root()?;
                Ok(cell.parser().load_address()?)
            }
            _ => Err(TonError::RunGetMethodResp(format!(
                "parse runGetMethod resp error"
            ))),
        }
    }
}
