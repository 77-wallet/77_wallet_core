use crate::tron::{
    Provider, consts,
    operations::{RawTransactionParams, TronTxOperation},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Reward {
    #[serde(default)]
    reward: i64,
}
impl Reward {
    pub fn to_sun(&self) -> f64 {
        self.reward as f64 / consts::TRX_TO_SUN as f64
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct WithdrawBalanceArgs {
    pub owner_address: String,
    // 提取的金额 unit is trx
    pub value: Option<f64>,
    #[serde(rename = "Permission_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_id: Option<i64>,
}

#[async_trait::async_trait]
impl TronTxOperation<WithdrawBalanceResp> for WithdrawBalanceArgs {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams> {
        let res = provider
            .withdraw_balance(&self.owner_address, self.permission_id)
            .await?;
        Ok(RawTransactionParams::from(res))
    }

    fn get_to(&self) -> String {
        String::new()
    }

    fn get_value(&self) -> f64 {
        self.value.unwrap_or_default()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct WithdrawBalanceResp {
    owner_address: String,
}

// 投票
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct VoteWitnessArgs {
    pub owner_address: String,
    pub votes: Vec<Votes>,
    pub visible: bool,
    #[serde(rename = "Permission_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_id: Option<i64>,
}
impl VoteWitnessArgs {
    pub fn new(
        owner_address: &str,
        votes: Vec<Votes>,
        permission: Option<i64>,
    ) -> crate::Result<Self> {
        Ok(Self {
            owner_address: wallet_utils::address::bs58_addr_to_hex(owner_address)?,
            votes,
            visible: false,
            permission_id: permission,
        })
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Votes {
    pub vote_address: String,
    pub vote_count: i64,
}

impl Votes {
    pub fn new(vote_address: &str, vote_count: i64) -> crate::Result<Self> {
        Ok(Self {
            vote_address: wallet_utils::address::bs58_addr_to_hex(vote_address)?,
            vote_count,
        })
    }
}

#[async_trait::async_trait]
impl TronTxOperation<VoteWitnessArgs> for VoteWitnessArgs {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams> {
        let res = provider.votes_wintess(self).await?;
        Ok(RawTransactionParams::from(res))
    }

    fn get_to(&self) -> String {
        String::new()
    }
    fn get_value(&self) -> f64 {
        0.0
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct VoteWitnessResp {
    pub owner_address: String,
    pub votes: Vec<Votes>,
    #[serde(default)]
    pub visible: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ListWitnessResp {
    pub witnesses: Vec<Witness>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Witness {
    pub address: String,
    #[serde(default)]
    pub vote_count: i64,
    pub url: String,
    total_produced: Option<i64>,
    total_missed: Option<i64>,
    latest_block_num: Option<i64>,
    latest_slot_num: Option<i64>,
    is_jobs: Option<bool>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct BrokerageResp {
    pub brokerage: i64,
}
