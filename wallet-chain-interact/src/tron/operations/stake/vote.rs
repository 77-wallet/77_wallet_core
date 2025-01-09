use crate::tron::{
    consts,
    operations::{RawTransactionParams, TronTxOperation},
    Provider,
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
}

#[async_trait::async_trait]
impl TronTxOperation<WithdrawBalanceResp> for WithdrawBalanceArgs {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams> {
        let res = provider.withdraw_balance(&self.owner_address).await?;
        Ok(RawTransactionParams::from(res))
    }

    fn get_to(&self) -> String {
        String::new()
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
}
impl VoteWitnessArgs {
    pub fn new(owner_address: &str, votes: Vec<Votes>) -> crate::Result<Self> {
        Ok(Self {
            owner_address: wallet_utils::address::bs58_addr_to_hex(owner_address)?,
            votes,
            visible: false,
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
