use bitcoin::{
    bip32::{ChildNumber, Xpriv},
    key::{Keypair, Secp256k1},
    Address, CompressedPublicKey, Network, PrivateKey,
};
use std::str::FromStr;
use wallet_chain_interact::btc::{provider::ProviderConfig, BtcChain};
use wallet_utils::init_test_log;
mod address;
mod base;
mod multisig;

fn main_config() -> ProviderConfig {
    let rpc_url = "rpc url".to_string();
    let http_url = "http url".to_string();

    ProviderConfig {
        rpc_url,
        http_url,
        rpc_auth: None,
        http_api_key: None,
    }
}

fn get_chain() -> BtcChain {
    init_test_log();

    let network = wallet_types::chain::network::NetworkKind::Regtest;
    // let config = get_local_config();
    let config = main_config();

    BtcChain::new(config, network, None).unwrap()
}

#[tokio::test]
async fn test_generate_address() {
    let secp = Secp256k1::new();

    let xprv_str = "private key";
    let xprv = Xpriv::from_str(xprv_str).expect("Invalid extended private key");

    let path = vec![
        ChildNumber::Hardened { index: 84 },
        ChildNumber::Hardened { index: 1 },
        ChildNumber::Hardened { index: 0 },
        ChildNumber::Normal { index: 0 },
    ];

    // 派生子私钥并生成地址
    for index in 0..5 {
        // 示例：生成前5个子私钥
        let child_number = ChildNumber::Normal { index };
        let child_key = xprv
            .derive_priv(&secp, &path)
            .expect("Failed to derive private key")
            .derive_priv(&secp, &vec![child_number])
            .expect("Failed to derive child private key");

        let private_key = PrivateKey {
            compressed: true,
            network: bitcoin::NetworkKind::Test,
            inner: child_key.private_key,
        };
        let wif = private_key.to_wif();

        // p2wpkh
        let c = CompressedPublicKey::from_private_key(&secp, &private_key).unwrap();
        // let address = Address::p2wpkh(&c, Network::Regtest);

        // p2pkh
        // let pubkey = private_key.public_key(&secp);
        // let address = Address::p2pkh(&pubkey, Network::Regtest);

        // p2sh-wpkh
        let address = Address::p2shwpkh(&c, Network::Testnet);

        // // taproot
        let _pubkey = Keypair::from_secret_key(&secp, &private_key.inner).x_only_public_key();
        // let address = Address::p2tr(&secp, pubkey.0, None, Network::Regtest);

        println!(
            "Child Private Key index = {} private_key = {} address = {}",
            index,
            wif,
            address.to_string()
        );
    }
}
