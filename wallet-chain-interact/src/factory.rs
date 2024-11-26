use crate::tron::{TronBlockChain, TronProvider};

pub struct ChainFactory;

impl ChainFactory {
    pub fn tron_chain(url: &str) -> crate::Result<TronBlockChain> {
        let provider = TronProvider::new(url)?;
        TronBlockChain::new(provider)
    }
}
