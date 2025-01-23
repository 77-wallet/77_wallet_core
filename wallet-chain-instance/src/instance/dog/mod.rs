pub(crate) mod address;
pub use address::generate_address_by_seckey;
mod test;

use std::str::FromStr;

use dogcoin::{
    bip32::{DerivationPath, Xpriv},
    key::Secp256k1,
};
use wallet_core::KeyPair;
use wallet_types::chain::{address::r#type::DogAddressType, chain::ChainCode, network};

// const NET: Network = Network::Testnet;

pub struct DogcoinKeyPair {
    dogcoin_family: ChainCode,
    pub xpriv: Xpriv,
    pubkey: String,
    address: String,
    derivation: DerivationPath,
    network: network::NetworkKind,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DogcoinInstance {
    pub(crate) chain_code: ChainCode,
    pub(crate) address_type: DogAddressType,
    pub network: network::NetworkKind,
}

impl wallet_core::derive::GenDerivationDog for DogcoinInstance {
    type Error = crate::Error;

    fn generate(
        address_type: &Option<DogAddressType>,
        input_index: i32,
    ) -> Result<String, crate::Error> {
        let path = if input_index < 0 {
            let i = wallet_utils::address::i32_index_to_unhardened_u32(input_index)?;
            let path = if let Some(address_type) = address_type {
                match address_type {
                    DogAddressType::P2pkh => wallet_types::constant::DOG_HARD_DERIVATION_PATH,
                    // DogAddressType::P2sh => todo!(),
                    // DogAddressType::P2shWpkh => todo!(),
                    DogAddressType::P2shWpkh => {
                        wallet_types::constant::DOG_SEG_WIT_HARD_DERIVATION_PATH
                    }
                    DogAddressType::P2wpkh => {
                        wallet_types::constant::DOG_SEG_WIT_NATIVE_HARD_DERIVATION_PATH
                    }
                    // DogAddressType::P2wsh => todo!(),
                    DogAddressType::P2tr => {
                        wallet_types::constant::DOG_TAPROOT_HARD_DERIVATION_PATH
                    }
                    // DogAddressType::P2trSh => todo!(),
                    _ => return Err(crate::Error::DogAddressTypeCantGenDerivationPath),
                }
            } else {
                return Err(wallet_types::Error::DogNeedAddressType.into());
            };
            crate::add_index(path, i, true)
        } else {
            let i = input_index as u32;
            let path = if let Some(address_type) = address_type {
                match address_type {
                    DogAddressType::P2pkh => wallet_types::constant::DOG_DERIVATION_PATH,
                    // DogAddressType::P2sh => todo!(),
                    // DogAddressType::P2shWpkh => todo!(),
                    DogAddressType::P2shWpkh => wallet_types::constant::DOG_SEG_WIT_DERIVATION_PATH,
                    DogAddressType::P2wpkh => {
                        wallet_types::constant::DOG_SEG_WIT_NATIVE_DERIVATION_PATH
                    }
                    // DogAddressType::P2wsh => todo!(),
                    DogAddressType::P2tr => wallet_types::constant::DOG_TAPROOT_DERIVATION_PATH,
                    // DogAddressType::P2trSh => todo!(),
                    _ => return Err(crate::Error::DogAddressTypeCantGenDerivationPath),
                }
            } else {
                return Err(wallet_types::Error::DogNeedAddressType.into());
            };
            crate::add_index(path, i, false)
        };

        // let res = crate::add_index(path, account_id);
        Ok(path)
    }
}

impl wallet_core::derive::Derive for DogcoinInstance {
    type Error = crate::Error;

    type Item = DogcoinKeyPair;

    // fn derive(&self, seed: Vec<u8>, index: u32) -> Result<Self::Item, Self::Error> {
    //     DogcoinKeyPair::generate(seed, index, &self.chain_code)
    // }

    fn derive_with_derivation_path(
        &self,
        seed: Vec<u8>,
        derivation_path: &str,
    ) -> Result<Self::Item, Self::Error> {
        let address =
            address::generate_address(&self.address_type, &seed, derivation_path, self.network)?;
        let mut res = DogcoinKeyPair::generate_with_derivation(
            seed,
            derivation_path,
            &self.chain_code,
            self.network,
        )?;
        res.address = address;
        Ok(res)
    }
}

impl KeyPair for DogcoinKeyPair {
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
        let prikey = dogcoin::PrivateKey::new(self.xpriv.private_key, network);
        Ok(prikey.to_string())
    }

    fn address(&self) -> String {
        self.address.clone()
    }
    fn pubkey(&self) -> String {
        self.pubkey.clone()
    }

    fn chain_code(&self) -> ChainCode {
        self.dogcoin_family
    }

    fn derivation_path(&self) -> String {
        let path = self.derivation.to_string();
        format!("m/{path}")
    }

    fn private_key_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        // let network = get_network(&self.chain_code())?;
        Ok(self.xpriv.private_key.secret_bytes().to_vec())
        // Ok(hex::decode(self.private_key()?).map_err(|e| crate::Error::Parse(e.into()))?)
    }
}

// fn get_network(chain_code: &Chain) -> Result<Network, crate::Error> {
//     let network = match chain_code {
//         Chain::Dog => Network::Dogcoin,
//         _ => return Err(wallet_core::Error::UnknownChain.into()),
//     };
//     Ok(network)
// }

fn generate(
    seed: Vec<u8>,
    derivation_path: &str,
    chain_code: &ChainCode,
    network: network::NetworkKind,
) -> Result<DogcoinKeyPair, crate::Error> {
    let xpriv = Xpriv::new_master(network, &seed).unwrap();

    // let pri_key = XPriv::root_from_seed(seed.as_slice(), None).unwrap();

    let path = DerivationPath::from_str(derivation_path).unwrap();
    let secp = Secp256k1::new();
    let derive_key = xpriv.derive_priv(&secp, &path).unwrap();

    // match path.

    // let derive = pri_key.derive_path(path.as_str()).unwrap();

    // let address = DogAddress {
    //     p2pkh: p2pkh.to_string(),
    //     p2sh: "".to_string(),
    //     p2wpkh: p2wpkh.to_string(),
    //     p2wpsh: "".to_string(),
    //     p2shwpkh: p2shwpkh.to_string(),
    //     p2shwsh: "".to_string(),
    //     p2tr: p2tr.to_string(),
    // };
    let keypair = derive_key.to_keypair(&secp);
    let pubkey = keypair.public_key().to_string();
    Ok(DogcoinKeyPair {
        dogcoin_family: chain_code.to_owned(),
        xpriv: derive_key,
        pubkey,
        address: "".to_string(),
        derivation: path,
        network,
    })
}
