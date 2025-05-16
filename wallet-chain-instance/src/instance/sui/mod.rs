pub mod address;
pub mod test;
use wallet_core::{KeyPair, derive::Derive};
use wallet_types::chain::{address::r#type::BtcAddressType, chain::ChainCode};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct SuiInstance {
    pub(crate) chain_code: ChainCode,
    pub network: wallet_types::chain::network::NetworkKind,
}

impl wallet_core::derive::GenDerivation for SuiInstance {
    type Error = crate::Error;
    fn generate(
        _address_type: &Option<BtcAddressType>,
        input_index: i32,
    ) -> Result<String, crate::Error> {
        let index = wallet_utils::address::i32_index_to_unhardened_u32(input_index)?;
        let path = crate::add_index(wallet_types::constant::SUI_DERIVATION_PATH, index, false);
        Ok(path)
    }
}

impl Derive for SuiInstance {
    type Error = crate::Error;
    type Item = SuiKeyPair;

    fn derive_with_derivation_path(
        &self,
        seed: Vec<u8>,
        derivation_path: &str,
    ) -> Result<Self::Item, Self::Error> {
        SuiKeyPair::generate_with_derivation(seed, derivation_path, &self.chain_code, self.network)
    }
}

pub struct SuiKeyPair {
    sui_family: ChainCode,
    private_key: String,
    pubkey: String,
    address: String,
    derivation: String,
    network: wallet_types::chain::network::NetworkKind,
}

impl KeyPair for SuiKeyPair {
    type Error = crate::Error;

    fn generate_with_derivation(
        seed: Vec<u8>,
        derivation_path: &str,
        chain_code: &ChainCode,
        network: wallet_types::chain::network::NetworkKind,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let pri_key = test::slip0010_derive_ed25519(&seed, derivation_path)?;
        let private_key = hex::encode(pri_key);

        let pub_key = test::get_pub_key(pri_key)?;
        let pubkey = hex::encode(pub_key.as_bytes());

        let address = test::generate_sui_address_from_bytes(pub_key.as_bytes());
        Ok(Self {
            sui_family: chain_code.to_owned(),
            private_key,
            pubkey,
            address,
            derivation: derivation_path.to_string(),
            network,
        })
    }

    fn network(&self) -> wallet_types::chain::network::NetworkKind {
        self.network
    }

    fn private_key(&self) -> Result<String, Self::Error> {
        Ok(self.private_key.clone())
    }
    fn pubkey(&self) -> String {
        self.pubkey.clone()
    }

    fn address(&self) -> String {
        self.address.clone()
    }

    fn derivation_path(&self) -> String {
        self.derivation.clone()
    }

    fn chain_code(&self) -> wallet_types::chain::chain::ChainCode {
        self.sui_family
    }

    fn private_key_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        Ok(wallet_utils::hex_func::hex_decode(&self.private_key)?)
    }
}
