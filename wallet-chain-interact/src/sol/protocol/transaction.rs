use crate::sol::operations::multisig::sods4_v4_has_create_account;
use serde::Deserialize;

pub enum CommitmentConfig {
    Processed,
    Confirmed,
    Finalized,
}
impl CommitmentConfig {
    pub fn to_string(&self) -> &'static str {
        match self {
            CommitmentConfig::Processed => "processed",
            CommitmentConfig::Confirmed => "confirmed",
            CommitmentConfig::Finalized => "finalized",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub block_time: u128,
    pub meta: Meta,
    pub slot: u128,
    pub transaction: Transaction,
}

impl TransactionResponse {
    // get all account about this transaction
    pub fn get_acccounts(&self) -> Vec<String> {
        let mut accounts = self.transaction.message.account_keys.clone();

        if !self.meta.loaded_addresses.writable.is_empty() {
            accounts.extend(self.meta.loaded_addresses.writable.clone());
        }
        if !self.meta.loaded_addresses.readonly.is_empty() {
            accounts.extend(self.meta.loaded_addresses.readonly.clone());
        }

        accounts
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransaction {
    pub meta: Meta,
    pub transaction: Transaction,
}
impl BlockTransaction {
    // get all account about this transaction
    pub fn get_acccounts(&self) -> Vec<String> {
        let mut accounts = self.transaction.message.account_keys.clone();

        if !self.meta.loaded_addresses.writable.is_empty() {
            accounts.extend(self.meta.loaded_addresses.writable.clone());
        }
        if !self.meta.loaded_addresses.readonly.is_empty() {
            accounts.extend(self.meta.loaded_addresses.readonly.clone());
        }

        accounts
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub compute_units_consumed: u128,
    pub fee: u64,
    pub status: Status,
    // pub log_messages: Vec<String>,
    pub loaded_addresses: LoadedAddresses,
    pub post_balances: Vec<u64>,
    pub post_token_balances: Vec<TokenBalance>,
    pub pre_balances: Vec<u64>,
    pub pre_token_balances: Vec<TokenBalance>,
}
impl Meta {
    // 可能创建了账号
    pub fn may_init_account(&self) -> std::collections::HashMap<usize, u64> {
        let mut result = std::collections::HashMap::new();

        for (index, &pre_balance) in self.pre_balances.iter().enumerate() {
            if pre_balance == 0 {
                if let Some(&post_balance) = self.post_balances.get(index) {
                    if post_balance > 0 {
                        result.insert(index, post_balance);
                    }
                }
            }
        }

        result
    }
}

#[derive(Debug, Deserialize)]
pub struct LoadedAddresses {
    pub writable: Vec<String>,
    pub readonly: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub account_index: u64,
    pub mint: String,
    pub owner: String,
    pub program_id: String,
    pub ui_token_amount: UiTokenAmount,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiTokenAmount {
    pub amount: String,
    pub decimals: u64,
    // pub ui_amount: Option<f64>,
    // pub ui_amount_string: String,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Status {
    Ok(Option<String>),
    Err(Option<serde_json::Value>),
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    pub message: Message,
    pub signatures: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub account_keys: Vec<String>,
    // pub address_table_lookups: Vec<AddressTableLookups>,
    // pub header: Vec<String>,
    pub instructions: Vec<TxInstruction>,
    pub recent_blockhash: String,
}

impl Message {
    pub fn has_create_account(&self, accounts: &[String]) -> bool {
        for instrction in self.instructions.iter() {
            if instrction.has_create_accoutn(accounts) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxInstruction {
    pub accounts: Vec<i32>,
    pub data: String,
    pub program_id_index: i32,
}

impl TxInstruction {
    pub fn has_create_accoutn(&self, accounts: &[String]) -> bool {
        let program_account = &accounts[self.program_id_index as usize];
        match program_account.as_str() {
            // token 账号 关联程序,调用了这个程序几乎创建了账号
            "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" => true,
            // 多签程序
            "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf" => {
                sods4_v4_has_create_account(&self.data)
            }
            // 其他
            _ => false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressTableLookups {
    pub account_key: Vec<i32>,
    pub readonly_indexes: Vec<i32>,
    pub writable_indexes: Vec<i32>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignatureStatus {
    pub slot: u64,
    pub confirmations: Option<i64>,
    pub confirmation_status: String,
    pub status: Status,
}
