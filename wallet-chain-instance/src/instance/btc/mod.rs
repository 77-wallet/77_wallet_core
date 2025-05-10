pub(crate) mod address;
pub use address::generate_address_by_seckey;

mod test;

use std::str::FromStr;

use bitcoin::{
    bip32::{DerivationPath, Xpriv},
    key::Secp256k1,
};
use wallet_core::KeyPair;
use wallet_types::chain::{address::r#type::BtcAddressType, chain::ChainCode, network};

pub struct BitcoinKeyPair {
    bitcoin_family: ChainCode,
    pub xpriv: Xpriv,
    pubkey: String,
    address: String,
    derivation: DerivationPath,
    network: network::NetworkKind,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BitcoinInstance {
    pub(crate) chain_code: ChainCode,
    pub(crate) address_type: BtcAddressType,
    pub network: network::NetworkKind,
}

impl wallet_core::derive::GenDerivation for BitcoinInstance {
    type Error = crate::Error;

    fn generate(
        address_type: &Option<BtcAddressType>,
        input_index: i32,
    ) -> Result<String, crate::Error> {
        let path = if input_index < 0 {
            let i = wallet_utils::address::i32_index_to_unhardened_u32(input_index)?;
            let path = if let Some(address_type) = address_type {
                match address_type {
                    BtcAddressType::P2pkh => wallet_types::constant::BTC_HARD_DERIVATION_PATH,
                    BtcAddressType::P2shWpkh => {
                        wallet_types::constant::BTC_SEG_WIT_HARD_DERIVATION_PATH
                    }
                    BtcAddressType::P2wpkh => {
                        wallet_types::constant::BTC_SEG_WIT_NATIVE_HARD_DERIVATION_PATH
                    }
                    BtcAddressType::P2tr => {
                        wallet_types::constant::BTC_TAPROOT_HARD_DERIVATION_PATH
                    }
                    _ => return Err(crate::Error::BtcAddressTypeCantGenDerivationPath),
                }
            } else {
                return Err(wallet_types::Error::BtcNeedAddressType.into());
            };
            crate::add_index(path, i, true)
        } else {
            let i = input_index as u32;
            let path = if let Some(address_type) = address_type {
                match address_type {
                    BtcAddressType::P2pkh => wallet_types::constant::BTC_DERIVATION_PATH,
                    BtcAddressType::P2shWpkh => wallet_types::constant::BTC_SEG_WIT_DERIVATION_PATH,
                    BtcAddressType::P2wpkh => {
                        wallet_types::constant::BTC_SEG_WIT_NATIVE_DERIVATION_PATH
                    }
                    BtcAddressType::P2tr => wallet_types::constant::BTC_TAPROOT_DERIVATION_PATH,
                    _ => return Err(crate::Error::BtcAddressTypeCantGenDerivationPath),
                }
            } else {
                return Err(wallet_types::Error::BtcNeedAddressType.into());
            };
            crate::add_index(path, i, false)
        };

        Ok(path)
    }
}

impl wallet_core::derive::Derive for BitcoinInstance {
    type Error = crate::Error;

    type Item = BitcoinKeyPair;

    fn derive_with_derivation_path(
        &self,
        seed: Vec<u8>,
        derivation_path: &str,
    ) -> Result<Self::Item, Self::Error> {
        let address =
            address::generate_address(&self.address_type, &seed, derivation_path, self.network)?;
        let mut res = BitcoinKeyPair::generate_with_derivation(
            seed,
            derivation_path,
            &self.chain_code,
            self.network,
        )?;
        res.address = address;
        Ok(res)
    }
}

impl KeyPair for BitcoinKeyPair {
    type Error = crate::Error;

    fn generate_with_derivation(
        seed: Vec<u8>,
        derivation_path: &str,
        chain_code: &ChainCode,
        network: network::NetworkKind,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        generate(seed, derivation_path, chain_code, network)
    }

    fn network(&self) -> network::NetworkKind {
        self.network
    }

    fn private_key(&self) -> Result<String, Self::Error> {
        let network = self.network();
        let prikey = bitcoin::PrivateKey::new(self.xpriv.private_key, network);
        Ok(prikey.to_string())
    }

    fn address(&self) -> String {
        self.address.clone()
    }
    fn pubkey(&self) -> String {
        self.pubkey.clone()
    }

    fn chain_code(&self) -> ChainCode {
        self.bitcoin_family
    }

    fn derivation_path(&self) -> String {
        let path = self.derivation.to_string();
        format!("m/{path}")
    }

    fn private_key_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        Ok(self.xpriv.private_key.secret_bytes().to_vec())
    }
}

fn generate(
    seed: Vec<u8>,
    derivation_path: &str,
    chain_code: &ChainCode,
    network: network::NetworkKind,
) -> Result<BitcoinKeyPair, crate::Error> {
    let xpriv = Xpriv::new_master(network, &seed).unwrap();

    let path = DerivationPath::from_str(derivation_path).unwrap();
    let secp = Secp256k1::new();
    let derive_key = xpriv.derive_priv(&secp, &path).unwrap();

    let keypair = derive_key.to_keypair(&secp);
    let pubkey = keypair.public_key().to_string();

    Ok(BitcoinKeyPair {
        bitcoin_family: chain_code.to_owned(),
        xpriv: derive_key,
        pubkey,
        address: "".to_string(),
        derivation: path,
        network,
    })
}
