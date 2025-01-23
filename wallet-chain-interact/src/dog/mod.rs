pub mod chain;
pub use chain::*;
use std::str::FromStr;
pub mod consts;
pub mod operations;
pub mod params;
pub mod protocol;
pub mod provider;
pub mod script;
pub mod signature;
// mod tx_build;
pub mod utxos;

pub struct ParseDogAddress {
    pub network: dogcoin::Network,
}
impl ParseDogAddress {
    pub fn new(network: wallet_types::chain::network::NetworkKind) -> Self {
        let network = network_convert(network);
        Self { network }
    }

    pub fn parse_address(&self, address: &str) -> crate::Result<dogcoin::Address> {
        let address = dogcoin::Address::from_str(address)
            .map_err(|e| {
                crate::ParseErr::AddressPraseErr(format!("err:{} address:{}", e, address))
            })?
            .require_network(self.network)
            .map_err(|e| {
                crate::ParseErr::AddressPraseErr(format!("err:{} address:{}", e, address))
            })?;
        Ok(address)
    }
}

pub fn network_convert(
    network: wallet_types::chain::network::NetworkKind,
) -> dogcoin::network::Network {
    match network {
        wallet_types::chain::network::NetworkKind::Regtest => dogcoin::network::Network::Regtest,
        wallet_types::chain::network::NetworkKind::Testnet => dogcoin::network::Network::Testnet,
        wallet_types::chain::network::NetworkKind::Mainnet => dogcoin::network::Network::Bitcoin,
    }
}

pub fn wif_private_key(
    bytes: &[u8],
    network: wallet_types::chain::network::NetworkKind,
) -> crate::Result<String> {
    let network = network_convert(network);
    Ok(dogcoin::PrivateKey::from_slice(bytes, network)
        .map_err(|e| crate::Error::SignError(e.to_string()))?
        .to_wif())
}
