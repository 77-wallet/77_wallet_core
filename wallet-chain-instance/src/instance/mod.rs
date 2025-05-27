pub mod btc;
pub mod dog;
pub mod eth;
pub mod ltc;
pub mod sol;
pub mod sui;
pub mod ton;
pub mod trx;

use std::fmt::Display;

use btc::BitcoinInstance;
use chain::ChainCode;
use dog::DogcoinInstance;
use eth::EthereumInstance;
use ltc::LitecoinInstance;
use sol::SolanaInstance;
use sui::SuiInstance;
use ton::{TonInstance, TonKeyPair};
use trx::TronInstance;
use wallet_core::{
    KeyPair,
    derive::{Derive, GenDerivation, GenDerivationDog, GenDerivationLtc},
};

use wallet_types::chain::{address::r#type::AddressType, chain, network};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum Address {
    EthAddress(alloy::primitives::Address),
    BtcAddress(String),
    LtcAddress(String),
    DogAddress(String),
    SolAddress(solana_sdk::pubkey::Pubkey),
    TrxAddress(anychain_tron::TronAddress),
    BnbAddress(alloy::primitives::Address),
    SuiAddress(String),
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Address::EthAddress(address) => write!(f, "{}", address),
            Address::BtcAddress(address) => write!(f, "{}", address),
            Address::LtcAddress(address) => write!(f, "{}", address),
            Address::DogAddress(address) => write!(f, "{}", address),
            Address::SolAddress(address) => write!(f, "{}", address),
            Address::TrxAddress(address) => write!(f, "{}", address.to_base58()),
            Address::BnbAddress(address) => write!(f, "{}", address),
            Address::SuiAddress(address) => write!(f, "{}", address),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ChainObject {
    Eth(crate::instance::eth::EthereumInstance),
    Trx(crate::instance::trx::TronInstance),
    Sol(crate::instance::sol::SolanaInstance),
    Bnb(crate::instance::eth::EthereumInstance),
    Btc(crate::instance::btc::BitcoinInstance),
    Ltc(crate::instance::ltc::LitecoinInstance),
    Dog(crate::instance::dog::DogcoinInstance),
    Ton(crate::instance::ton::TonInstance),
    Sui(crate::instance::sui::SuiInstance),
}

impl ChainObject {
    pub fn new(
        chain_code: &str,
        address_type: Option<String>,
        network: network::NetworkKind,
    ) -> Result<Self, crate::Error> {
        let chain_code: ChainCode = chain_code.try_into()?;
        let btc_address_type: AddressType = address_type.try_into()?;
        (&chain_code, &btc_address_type, network).try_into()
    }

    pub fn chain_code(&self) -> &ChainCode {
        match self {
            ChainObject::Eth(i) => &i.chain_code,
            ChainObject::Trx(i) => &i.chain_code,
            ChainObject::Sol(i) => &i.chain_code,
            ChainObject::Bnb(i) => &i.chain_code,
            ChainObject::Btc(i) => &i.chain_code,
            ChainObject::Ltc(i) => &i.chain_code,
            ChainObject::Dog(i) => &i.chain_code,
            ChainObject::Ton(i) => &i.chain_code,
            ChainObject::Sui(i) => &i.chain_code,
        }
    }

    pub fn address_type(&self) -> AddressType {
        match self {
            ChainObject::Eth(_)
            | ChainObject::Trx(_)
            | ChainObject::Sol(_)
            | ChainObject::Bnb(_)
            | ChainObject::Sui(_) => AddressType::Other,
            ChainObject::Btc(i) => AddressType::Btc(i.address_type),
            ChainObject::Ltc(i) => AddressType::Ltc(i.address_type),
            ChainObject::Dog(i) => AddressType::Dog(i.address_type),
            ChainObject::Ton(i) => AddressType::Ton(i.address_type),
        }
    }

    pub fn gen_keypair_with_index_address_type(
        &self,
        seed: &[u8],
        input_index: i32,
    ) -> Result<Box<dyn wallet_core::KeyPair<Error = crate::Error>>, crate::Error> {
        match self {
            ChainObject::Eth(i) => {
                let derivation_path = EthereumInstance::generate(&None, input_index)?;
                let res = i.derive_with_derivation_path(seed.to_vec(), &derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Trx(i) => {
                let derivation_path = TronInstance::generate(&None, input_index)?;
                let res = i.derive_with_derivation_path(seed.to_vec(), &derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Sol(i) => {
                let derivation_path = SolanaInstance::generate(&None, input_index)?;
                let res = i.derive_with_derivation_path(seed.to_vec(), &derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Bnb(i) => {
                let derivation_path = EthereumInstance::generate(&None, input_index)?;
                let res = i.derive_with_derivation_path(seed.to_vec(), &derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Btc(i) => {
                let derivation_path =
                    BitcoinInstance::generate(&Some(i.address_type), input_index)?;
                let res = i.derive_with_derivation_path(seed.to_vec(), &derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Ltc(i) => {
                let derivation_path =
                    LitecoinInstance::generate(&Some(i.address_type), input_index)?;
                let res = i.derive_with_derivation_path(seed.to_vec(), &derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Dog(i) => {
                let derivation_path =
                    DogcoinInstance::generate(&Some(i.address_type), input_index)?;
                let res = i.derive_with_derivation_path(seed.to_vec(), &derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }

            ChainObject::Ton(instance) => {
                let derivation_path = TonInstance::generate(&None, input_index)?;
                let res = TonKeyPair::generate_with_derivation(
                    seed.to_vec(),
                    &derivation_path,
                    &instance.chain_code,
                    instance.network,
                )?;

                Ok(Box::new(res))
            }
            ChainObject::Sui(i) => {
                let derivation_path = SuiInstance::generate(&None, input_index)?;
                let res = i.derive_with_derivation_path(seed.to_vec(), &derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
        }
    }

    pub fn gen_keypair_with_derivation_path(
        &self,
        seed: &[u8],
        derivation_path: &str,
    ) -> Result<Box<dyn wallet_core::KeyPair<Error = crate::Error>>, crate::Error> {
        match self {
            ChainObject::Eth(i) => {
                let res = i.derive_with_derivation_path(seed.to_vec(), derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Trx(i) => {
                let res = i.derive_with_derivation_path(seed.to_vec(), derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Sol(i) => {
                let res = i.derive_with_derivation_path(seed.to_vec(), derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Bnb(i) => {
                let res = i.derive_with_derivation_path(seed.to_vec(), derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Btc(i) => {
                let res = i.derive_with_derivation_path(seed.to_vec(), derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Ltc(i) => {
                let res = i.derive_with_derivation_path(seed.to_vec(), derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Dog(i) => {
                let res = i.derive_with_derivation_path(seed.to_vec(), derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
            ChainObject::Ton(instance) => {
                let res = TonKeyPair::generate_with_derivation(
                    seed.to_vec(),
                    derivation_path,
                    &instance.chain_code,
                    instance.network,
                )?;

                Ok(Box::new(res))
            }
            ChainObject::Sui(i) => {
                let res = i.derive_with_derivation_path(seed.to_vec(), derivation_path)?;
                let res = Box::new(res);
                Ok(res)
            }
        }
    }

    pub fn gen_gen_address(
        &self,
    ) -> Result<
        Box<
            dyn wallet_core::address::GenAddress<
                    Address = crate::instance::Address,
                    Error = crate::Error,
                >,
        >,
        crate::Error,
    > {
        Ok(match self {
            ChainObject::Eth(_) => Box::new(crate::instance::eth::address::EthGenAddress::new(
                chain::ChainCode::Ethereum,
            )),
            ChainObject::Trx(_) => Box::new(crate::instance::trx::address::TrxGenAddress {}),
            ChainObject::Sol(_) => Box::new(crate::instance::sol::address::SolGenAddress {}),
            ChainObject::Bnb(_) => Box::new(crate::instance::eth::address::EthGenAddress::new(
                chain::ChainCode::BnbSmartChain,
            )),
            ChainObject::Btc(i) => Box::new(crate::instance::btc::address::BtcGenAddress {
                address_type: i.address_type,
                network: i.network,
            }),
            ChainObject::Ltc(i) => Box::new(crate::instance::ltc::address::LtcGenAddress {
                address_type: i.address_type,
                network: i.network,
            }),
            ChainObject::Dog(i) => Box::new(crate::instance::dog::address::DogGenAddress {
                address_type: i.address_type,
                network: i.network,
            }),
            ChainObject::Sui(_) => Box::new(crate::instance::sui::address::SuiGenAddress {}),
            _ => panic!("not suer used"),
        })
    }
}

impl TryFrom<(&ChainCode, &AddressType, network::NetworkKind)> for ChainObject {
    type Error = crate::Error;

    fn try_from(
        (value, typ, network): (&ChainCode, &AddressType, network::NetworkKind),
    ) -> Result<Self, Self::Error> {
        let res = match value {
            ChainCode::Ethereum => ChainObject::Eth(crate::instance::eth::EthereumInstance {
                chain_code: value.to_owned(),
                network,
            }),
            ChainCode::Tron => ChainObject::Trx(crate::instance::trx::TronInstance {
                chain_code: value.to_owned(),
                network,
            }),
            ChainCode::Solana => ChainObject::Sol(crate::instance::sol::SolanaInstance {
                chain_code: value.to_owned(),
                network,
            }),
            ChainCode::BnbSmartChain => ChainObject::Bnb(crate::instance::eth::EthereumInstance {
                chain_code: value.to_owned(),
                network,
            }),
            ChainCode::Bitcoin => {
                let AddressType::Btc(btc_address_type) = typ else {
                    return Err(crate::Error::Types(wallet_types::Error::BtcNeedAddressType));
                };

                ChainObject::Btc(crate::instance::btc::BitcoinInstance {
                    chain_code: value.to_owned(),
                    address_type: btc_address_type.to_owned(),
                    network,
                })
            }
            ChainCode::Litecoin => {
                let AddressType::Ltc(ltc) = typ else {
                    return Err(crate::Error::Types(wallet_types::Error::LtcNeedAddressType));
                };

                ChainObject::Ltc(crate::instance::ltc::LitecoinInstance {
                    chain_code: value.to_owned(),
                    address_type: ltc.to_owned(),
                    network,
                })
            }
            ChainCode::Dogcoin => {
                let AddressType::Dog(doge) = typ else {
                    return Err(crate::Error::Types(wallet_types::Error::DogNeedAddressType));
                };

                ChainObject::Dog(crate::instance::dog::DogcoinInstance {
                    chain_code: value.to_owned(),
                    address_type: doge.to_owned(),
                    network,
                })
            }
            ChainCode::Ton => {
                let AddressType::Ton(ton) = typ else {
                    return Err(crate::Error::Types(wallet_types::Error::MissAddressType));
                };

                ChainObject::Ton(crate::instance::ton::TonInstance {
                    chain_code: value.clone(),
                    address_type: ton.to_owned(),
                    network,
                })
            }
            ChainCode::Sui => ChainObject::Sui(crate::instance::sui::SuiInstance {
                chain_code: value.to_owned(),
                network,
            }),
        };
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::ChainObject;
    use wallet_core::xpriv;
    use wallet_types::chain::{address::r#type::DOG_ADDRESS_TYPES, chain::ChainCode, network};

    #[test]
    fn test_gen() {
        let phrase = "";
        let password = "";

        let xpriv = xpriv::generate_master_key(1, phrase, password).unwrap();
        let seed = xpriv.1;

        let code = ChainCode::Dogcoin;
        let address_types = DOG_ADDRESS_TYPES.to_vec();
        let network = network::NetworkKind::Testnet;

        for address_type in address_types {
            let instance: ChainObject = (&code, &address_type, network.into()).try_into().unwrap();
            let keypair = instance
                .gen_keypair_with_index_address_type(&seed, 0)
                .unwrap();

            println!("address {}", keypair.address())
        }
    }
}
