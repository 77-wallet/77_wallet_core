use crate::{
    eth::protocol::contract::{createProxyWithNonceCall, setupCall},
    types,
};
use alloy::{
    network::TransactionBuilder, primitives, rpc::types::TransactionRequest, sol_types::SolCall,
};
use wallet_utils::{address, unit};

pub struct MultisigAccountOpt {
    pub from: primitives::Address,
    pub threshold: i32,
    pub owners: Vec<primitives::Address>,
    pub nonce: Option<primitives::U256>,
}

impl MultisigAccountOpt {
    pub fn new(from: &str, threshold: i32) -> Result<Self, crate::Error> {
        let from = address::parse_eth_address(from)?;
        Ok(Self {
            from,
            threshold,
            owners: vec![],
            nonce: None,
        })
    }

    pub fn set_nonce(mut self, nonce: &str) -> Result<Self, crate::Error> {
        let nonce = unit::u256_from_str(nonce)?;
        self.nonce = Some(nonce);
        Ok(self)
    }

    pub fn with_nonce(mut self) -> Self {
        use bitcoin::key::rand::Rng;
        let mut rand = bitcoin::key::rand::thread_rng();

        self.nonce = Some(primitives::U256::from(Rng::gen::<u64>(&mut rand)));
        self
    }

    pub fn with_owners(mut self, owners: Vec<String>) -> Result<Self, crate::Error> {
        let mut o = vec![];
        for owner in owners {
            let address = address::parse_eth_address(&owner)?;
            o.push(address);
        }

        self.owners = o;
        Ok(self)
    }

    pub fn get_nonce(&self) -> crate::Result<primitives::U256> {
        match self.nonce {
            Some(nonce) => Ok(nonce),
            None => Err(crate::Error::Other("nonce is None".to_string())),
        }
    }
}

impl types::Transaction<TransactionRequest> for MultisigAccountOpt {
    fn build_transaction(&self) -> Result<TransactionRequest, crate::Error> {
        let factory_addr = address::parse_eth_address(super::MULTISIG_FACTORY)?;
        let safe_addr = address::parse_eth_address(super::SAFE)?;

        let default_address = primitives::Address::default();
        let set_up = setupCall {
            _owners: self.owners.clone(),
            _threshold: primitives::U256::from(self.threshold),
            to: default_address,
            data: primitives::Bytes::default(),
            fallbackHandler: default_address,
            paymentToken: default_address,
            payment: primitives::U256::ZERO,
            paymentReceiver: default_address,
        };

        let create = createProxyWithNonceCall {
            _singleton: safe_addr,
            initializer: set_up.abi_encode().into(),
            saltNonce: self.get_nonce()?,
        };

        Ok(TransactionRequest::default()
            .with_from(self.from)
            .with_to(factory_addr)
            .with_value(primitives::U256::ZERO)
            .with_input(create.abi_encode()))
    }
}
