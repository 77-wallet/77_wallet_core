pub mod address;
use coins_bip32::xkeys::XPriv;
use wallet_core::KeyPair;
use wallet_types::chain::{address::r#type::BtcAddressType, chain::ChainCode};

pub struct EthereumKeyPair {
    // master_key: XPriv,
    ethereum_family: ChainCode,
    private_key: XPriv,
    pubkey: String,
    address: alloy::primitives::Address,
    derivation: String,
    network: wallet_types::chain::network::NetworkKind,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct EthereumInstance {
    pub chain_code: ChainCode,
    pub network: wallet_types::chain::network::NetworkKind,
}

impl wallet_core::derive::GenDerivation for EthereumInstance {
    type Error = crate::Error;
    fn generate(
        _address_type: &Option<BtcAddressType>,
        input_index: i32,
    ) -> Result<String, crate::Error> {
        let path = if input_index < 0 {
            let i = wallet_utils::address::i32_index_to_unhardened_u32(input_index)?;
            crate::add_index(wallet_types::constant::ETH_HARD_DERIVATION_PATH, i, true)
        } else {
            let i = input_index as u32;
            crate::add_index(wallet_types::constant::ETH_DERIVATION_PATH, i, false)
        };
        Ok(path)
    }
}

impl wallet_core::derive::Derive for EthereumInstance {
    type Error = crate::Error;
    type Item = EthereumKeyPair;

    // fn derive(&self, seed: Vec<u8>, index: u32) -> Result<EthereumKeyPair, Self::Error> {
    //     EthereumKeyPair::generate(seed, index, &self.chain_code)
    // }

    fn derive_with_derivation_path(
        &self,
        seed: Vec<u8>,
        derivation_path: &str,
    ) -> Result<Self::Item, Self::Error> {
        EthereumKeyPair::generate_with_derivation(
            seed,
            derivation_path,
            &self.chain_code,
            self.network,
        )
    }
}

impl wallet_core::KeyPair for EthereumKeyPair {
    type Error = crate::Error;
    // type Raw = coins_bip32::xkeys::XPriv;
    // type Address = alloy::primitives::Address;
    // type PrivateKey = XPriv;

    // fn generate(seed: Vec<u8>, index: u32, chain_code: &Chain) -> Result<Self, Self::Error>
    // where
    //     Self: Sized,
    // {
    //     let pri_key = XPriv::root_from_seed(seed.as_slice(), None).unwrap();
    //     let path =
    //         wallet_core::constant::add_index(wallet_core::constant::ETH_DERIVATION_PATH, index);
    //     tracing::error!("[generate] path: {path}");
    //     let derive = pri_key.derive_path(path.as_str()).unwrap();
    //     let signingkey: &coins_bip32::ecdsa::SigningKey = derive.as_ref();
    //     let address = alloy::signers::utils::secret_key_to_address(signingkey);
    //     Ok(Self {
    //         // master_key: pri_key,
    //         ethereum_family: chain_code.to_owned(),
    //         private_key: derive,
    //         address,
    //         derivation: path,
    //     })
    // }
    fn network(&self) -> wallet_types::chain::network::NetworkKind {
        self.network
    }
    fn generate_with_derivation(
        seed: Vec<u8>,
        derivation_path: &str,
        chain_code: &ChainCode,
        network: wallet_types::chain::network::NetworkKind,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let pri_key = XPriv::root_from_seed(seed.as_slice(), None).unwrap();
        let derive = pri_key.derive_path(derivation_path).unwrap();
        let signingkey: &coins_bip32::ecdsa::SigningKey = derive.as_ref();

        let verif_key = signingkey.verifying_key().to_sec1_bytes();

        let pubkey = wallet_utils::hex_func::hex_encode(verif_key);
        let address = alloy::signers::utils::secret_key_to_address(signingkey);

        Ok(Self {
            ethereum_family: chain_code.to_owned(),
            private_key: derive,
            pubkey,
            address,
            derivation: derivation_path.to_string(),
            network,
        })
    }

    fn private_key(&self) -> Result<String, Self::Error> {
        let signingkey: &coins_bip32::ecdsa::SigningKey = self.private_key.as_ref();
        let private_key = signingkey.to_bytes();

        Ok(hex::encode(private_key))
    }

    fn address(&self) -> String {
        self.address.to_string()
    }
    fn pubkey(&self) -> String {
        self.pubkey.clone()
    }

    fn derivation_path(&self) -> String {
        self.derivation.clone()
    }

    fn chain_code(&self) -> ChainCode {
        self.ethereum_family
    }

    fn private_key_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        let signingkey: &coins_bip32::ecdsa::SigningKey = self.private_key.as_ref();
        let private_key = signingkey.to_bytes();
        Ok(private_key.to_vec())
        // Ok(hex::decode(self.private_key()?).map_err(|e| crate::Error::Parse(e.into()))?)
    }

    // fn master_key(&self) -> Result<coins_bip32::xkeys::XPriv, Self::Error> {
    //     let pri_key = XPriv::root_from_seed(seed.as_slice(), None).unwrap();
    // }
}
