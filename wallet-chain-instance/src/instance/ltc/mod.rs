pub(crate) mod address;
pub use address::generate_address_by_seckey;
mod test;

use std::str::FromStr;

use litecoin::{
    bip32::{DerivationPath, Xpriv},
    key::Secp256k1,
};
use wallet_core::KeyPair;
use wallet_types::chain::{address::r#type::LtcAddressType, chain::ChainCode, network};

// const NET: Network = Network::Testnet;

pub struct LitecoinKeyPair {
    litecoin_family: ChainCode,
    pub xpriv: Xpriv,
    pubkey: String,
    address: String,
    derivation: DerivationPath,
    network: network::NetworkKind,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LitecoinInstance {
    pub(crate) chain_code: ChainCode,
    pub(crate) address_type: LtcAddressType,
    pub network: network::NetworkKind,
}

impl wallet_core::derive::GenDerivationLtc for LitecoinInstance {
    type Error = crate::Error;

    fn generate(
        address_type: &Option<LtcAddressType>,
        input_index: i32,
    ) -> Result<String, crate::Error> {
        let path = if input_index < 0 {
            let i = wallet_utils::address::i32_index_to_unhardened_u32(input_index)?;
            let path = if let Some(address_type) = address_type {
                match address_type {
                    LtcAddressType::P2pkh => wallet_types::constant::LTC_HARD_DERIVATION_PATH,
                    // LtcAddressType::P2sh => todo!(),
                    // LtcAddressType::P2shWpkh => todo!(),
                    LtcAddressType::P2shWpkh => {
                        wallet_types::constant::LTC_SEG_WIT_HARD_DERIVATION_PATH
                    }
                    LtcAddressType::P2wpkh => {
                        wallet_types::constant::LTC_SEG_WIT_NATIVE_HARD_DERIVATION_PATH
                    }
                    // LtcAddressType::P2wsh => todo!(),
                    LtcAddressType::P2tr => {
                        wallet_types::constant::LTC_TAPROOT_HARD_DERIVATION_PATH
                    }
                    // LtcAddressType::P2trSh => todo!(),
                    _ => return Err(crate::Error::LtcAddressTypeCantGenDerivationPath),
                }
            } else {
                return Err(wallet_types::Error::LtcNeedAddressType.into());
            };
            crate::add_index(path, i, true)
        } else {
            let i = input_index as u32;
            let path = if let Some(address_type) = address_type {
                match address_type {
                    LtcAddressType::P2pkh => wallet_types::constant::LTC_DERIVATION_PATH,
                    // LtcAddressType::P2sh => todo!(),
                    // LtcAddressType::P2shWpkh => todo!(),
                    LtcAddressType::P2shWpkh => wallet_types::constant::LTC_SEG_WIT_DERIVATION_PATH,
                    LtcAddressType::P2wpkh => {
                        wallet_types::constant::LTC_SEG_WIT_NATIVE_DERIVATION_PATH
                    }
                    // LtcAddressType::P2wsh => todo!(),
                    LtcAddressType::P2tr => wallet_types::constant::LTC_TAPROOT_DERIVATION_PATH,
                    // LtcAddressType::P2trSh => todo!(),
                    _ => return Err(crate::Error::LtcAddressTypeCantGenDerivationPath),
                }
            } else {
                return Err(wallet_types::Error::LtcNeedAddressType.into());
            };
            crate::add_index(path, i, false)
        };

        // let res = crate::add_index(path, account_id);
        Ok(path)
    }
}

impl wallet_core::derive::Derive for LitecoinInstance {
    type Error = crate::Error;

    type Item = LitecoinKeyPair;

    // fn derive(&self, seed: Vec<u8>, index: u32) -> Result<Self::Item, Self::Error> {
    //     LitecoinKeyPair::generate(seed, index, &self.chain_code)
    // }

    fn derive_with_derivation_path(
        &self,
        seed: Vec<u8>,
        derivation_path: &str,
    ) -> Result<Self::Item, Self::Error> {
        let address =
            address::generate_address(&self.address_type, &seed, derivation_path, self.network)?;
        let mut res = LitecoinKeyPair::generate_with_derivation(
            seed,
            derivation_path,
            &self.chain_code,
            self.network,
        )?;
        res.address = address;
        Ok(res)
    }
}

impl KeyPair for LitecoinKeyPair {
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
        let prikey = litecoin::PrivateKey::new(self.xpriv.private_key, network);
        Ok(prikey.to_string())
    }

    fn address(&self) -> String {
        self.address.clone()
    }
    fn pubkey(&self) -> String {
        self.pubkey.clone()
    }

    fn chain_code(&self) -> ChainCode {
        self.litecoin_family
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
//         Chain::Ltc => Network::Litecoin,
//         _ => return Err(wallet_core::Error::UnknownChain.into()),
//     };
//     Ok(network)
// }

fn generate(
    seed: Vec<u8>,
    derivation_path: &str,
    chain_code: &ChainCode,
    network: network::NetworkKind,
) -> Result<LitecoinKeyPair, crate::Error> {
    let xpriv = Xpriv::new_master(network, &seed).unwrap();

    // let pri_key = XPriv::root_from_seed(seed.as_slice(), None).unwrap();

    let path = DerivationPath::from_str(derivation_path).unwrap();
    let secp = Secp256k1::new();
    let derive_key = xpriv.derive_priv(&secp, &path).unwrap();

    // match path.

    // let derive = pri_key.derive_path(path.as_str()).unwrap();

    // let address = LitecoinAddress {
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
    Ok(LitecoinKeyPair {
        litecoin_family: chain_code.to_owned(),
        xpriv: derive_key,
        pubkey,
        address: "".to_string(),
        derivation: path,
        network,
    })
}
