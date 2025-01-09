use super::{RawTransactionParams, TronTransactionResponse, TronTxOperation};
use crate::{
    tron::{consts::PERMISSION, provider::Provider},
    types::{ChainPrivateKey, MultisigSignResp},
};
use serde_json::json;
use std::{fmt::Debug, str::FromStr as _};
use wallet_utils::{address, sign};

pub struct MultisigAccountOpt {
    pub from: String,
    pub threshold: u8,
    pub owners: Vec<String>,
}

impl MultisigAccountOpt {
    pub fn new(from: &str, threshold: u8, owners: Vec<String>) -> crate::Result<Self> {
        let mut owner = vec![];
        for o in owners {
            owner.push(address::bs58_addr_to_hex(&o)?);
        }

        Ok(Self {
            from: address::bs58_addr_to_hex(from)?,
            threshold,
            owners: owner,
        })
    }

    pub fn owner_to_keys(&self, weight: i8) -> Vec<Keys> {
        self.owners
            .iter()
            .map(|item| Keys {
                address: item.clone(),
                weight,
            })
            .collect()
    }
}
impl From<&MultisigAccountOpt> for MultisigAccountResp {
    fn from(value: &MultisigAccountOpt) -> Self {
        let owner = Permission::new(value.threshold, value.owner_to_keys(1));

        let mut actives = owner.clone();
        actives.types = Some(json!(2));
        actives.permission_name = "active0".to_string();
        actives.operations = Some(PERMISSION.to_string());

        Self {
            owner_address: value.from.clone(),
            owner,
            actives: vec![actives],
            visible: None,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MultisigAccountResp {
    owner_address: String,
    owner: Permission,
    actives: Vec<Permission>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visible: Option<bool>,
}

#[async_trait::async_trait]
impl TronTxOperation<MultisigAccountResp> for MultisigAccountOpt {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams> {
        let params = MultisigAccountResp::from(self);

        let res: TronTransactionResponse<MultisigAccountResp> = provider
            .do_request("wallet/accountpermissionupdate", Some(params))
            .await?;
        Ok(RawTransactionParams::from(res))
    }
    fn get_to(&self) -> String {
        self.from.clone()
    }

    fn get_value(&self) -> i64 {
        0
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Permission {
    #[serde(rename = "type")]
    pub types: Option<serde_json::Value>,
    pub permission_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<String>,
    pub threshold: u8,
    pub keys: Vec<Keys>,
}
impl Permission {
    pub fn new(threshold: u8, keys: Vec<Keys>) -> Self {
        Self {
            types: Some(json!(0)),
            permission_name: "owner".to_owned(),
            operations: None,
            threshold,
            keys,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Keys {
    address: String,
    weight: i8,
}

pub struct TransactionOpt;
impl TransactionOpt {
    pub fn data_from_str(data: &str) -> crate::Result<RawTransactionParams> {
        RawTransactionParams::from_str(data)
    }

    pub fn sign_transaction(
        raw_data: &str,
        key: ChainPrivateKey,
    ) -> crate::Result<MultisigSignResp> {
        let data = TransactionOpt::data_from_str(raw_data)?;

        let signature = sign::sign_tron(&data.tx_id, &key, None)?;

        Ok(MultisigSignResp {
            tx_hash: data.tx_id,
            signature,
        })
    }
}
