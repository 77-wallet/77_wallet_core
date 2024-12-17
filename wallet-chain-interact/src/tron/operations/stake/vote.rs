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
pub struct WithdrawBalance {
    pub owner_address: String,
}

#[async_trait::async_trait]
impl TronTxOperation<WithdrawBalanceResp> for WithdrawBalance {
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
pub struct VoteWitness {
    pub owner_address: String,
    pub votes: Vec<Votes>,
}
impl VoteWitness {
    pub fn new(owner_address: &str, votes: Vec<Votes>) -> crate::Result<Self> {
        Ok(Self {
            owner_address: wallet_utils::address::bs58_addr_to_hex(owner_address)?,
            votes,
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
impl TronTxOperation<VoteWitness> for VoteWitness {
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
