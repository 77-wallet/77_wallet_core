use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AccountInfo {
    pub data: Vec<String>,
    pub executable: bool,
    pub lamports: u64,
    pub owner: String,
    #[serde(rename = "rentEpoch")]
    pub rent_epoch: u64,
    pub space: u64,
}

#[derive(Debug, Deserialize)]
pub struct Balance {
    pub context: Context,
    pub value: u128,
}

#[derive(Debug, Deserialize)]
pub struct Context {
    pub slot: u128,
}

#[derive(Debug, Deserialize)]
pub struct TokenAccount {
    pub context: Context,
    pub value: Vec<TokenAccountValue>,
}
impl TokenAccount {
    pub fn balance(&self) -> String {
        if self.value.is_empty() {
            return "0".to_string();
        }
        self.value
            .first()
            .unwrap()
            .account
            .data
            .parsed
            .info
            .token_amount
            .amount
            .clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct TokenAccountValue {
    pub account: Account,
    pub pubkey: String,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    pub data: AccountData,
    pub executable: bool,
    pub lamports: u64,
    pub owner: String,
    #[serde(rename = "rentEpoch")]
    pub rent_epoch: u64,
    pub space: u64,
}

#[derive(Debug, Deserialize)]
pub struct AccountData {
    pub parsed: Parsed,
    pub program: String,
    pub space: u64,
}

#[derive(Debug, Deserialize)]
pub struct Parsed {
    pub info: Info,
    #[serde(rename = "type")]
    pub types: String,
}

#[derive(Debug, Deserialize)]
pub struct Info {
    #[serde(rename = "isNative")]
    pub is_native: bool,
    pub mint: String,
    pub owner: String,
    pub state: String,
    #[serde(rename = "tokenAmount")]
    pub token_amount: TokenAmount,
}

#[derive(Debug, Deserialize)]
pub struct TokenAmount {
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: f64,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}
