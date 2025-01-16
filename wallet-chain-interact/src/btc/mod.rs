pub mod chain;
use std::str::FromStr;

pub use chain::*;
pub mod consts;
pub mod operations;
pub mod params;
pub mod protocol;
pub mod provider;
pub mod script;
mod signature;
pub use signature::MultisigSignParams;
// mod tx_build;
mod utxos;

pub struct ParseBtcAddress {
    pub network: bitcoin::Network,
}
impl ParseBtcAddress {
    pub fn new(network: wallet_types::chain::network::NetworkKind) -> Self {
        let network = network_convert(network);
        Self { network }
    }

    pub fn parse_address(&self, address: &str) -> crate::Result<bitcoin::Address> {
        let address = bitcoin::Address::from_str(address)
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
) -> bitcoin::network::Network {
    match network {
        wallet_types::chain::network::NetworkKind::Regtest => bitcoin::network::Network::Regtest,
        wallet_types::chain::network::NetworkKind::Testnet => bitcoin::network::Network::Testnet,
        wallet_types::chain::network::NetworkKind::Mainnet => bitcoin::network::Network::Bitcoin,
    }
}

pub fn wif_private_key(
    bytes: &[u8],
    network: wallet_types::chain::network::NetworkKind,
) -> crate::Result<String> {
    let network = network_convert(network);
    Ok(bitcoin::PrivateKey::from_slice(bytes, network)
        .map_err(|e| crate::Error::SignError(e.to_string()))?
        .to_wif())
}
