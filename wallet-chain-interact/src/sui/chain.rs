use super::provider::Provider;
use wallet_types::chain::chain::ChainCode;
use wallet_types::chain::network;

pub struct SuiChain {
    pub provider: Provider,
    chain_code: ChainCode,
    network: network::NetworkKind,
}

impl SuiChain {
    pub fn new(
        provider: Provider,
        network: network::NetworkKind,
        chain_code: ChainCode,
    ) -> crate::Result<Self> {
        Ok(Self {
            provider,
            chain_code,
            network,
        })
    }
}

impl SuiChain {
    pub async fn balance(
        &self,
        addr: &str,
        token: Option<String>,
    ) -> crate::Result<sui_sdk::rpc_types::Balance> {
        if let Some(t) = token
            && !t.is_empty()
        {
            self.provider.token_balance(addr, &t).await
        } else {
            self.provider.balance(addr).await
        }
    }
}

impl SuiChain {}
