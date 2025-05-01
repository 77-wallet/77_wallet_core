use super::provider::Provider;
use alloy::primitives::U256;

pub struct TonChain {
    pub provider: Provider,
}
impl TonChain {
    pub fn new(provider: Provider) -> crate::Result<Self> {
        Ok(Self { provider })
    }
}

impl TonChain {
    pub async fn balance(&self, addr: &str, _token: Option<String>) -> crate::Result<U256> {
        // if let Some(t) = token
        //     && !t.is_empty()
        // {
        // } else {
        // }

        self.provider.balance(addr).await
    }
}
