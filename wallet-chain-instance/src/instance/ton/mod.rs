use dogcoin::hex::DisplayHex;
use ed25519_dalek_bip32::{DerivationPath, ExtendedSecretKey};
use std::str::FromStr;
use tonlib_core::wallet::{
    mnemonic::KeyPair, ton_wallet::TonWallet, wallet_version::WalletVersion,
};
use wallet_types::chain::{
    address::r#type::{BtcAddressType, TonAddressType},
    chain::ChainCode,
};
pub struct TonKeyPair {
    tron_family: ChainCode,
    private_key: ExtendedSecretKey,
    network: wallet_types::chain::network::NetworkKind,
    derivation: String,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct TonInstance {
    pub(crate) chain_code: ChainCode,
    pub network: wallet_types::chain::network::NetworkKind,
    pub address_type: TonAddressType,
}

impl TonInstance {
    pub const TON_DERIVATION_PATH: &'static str = "m/44'/607'/0'";

    pub fn address_from_private_key(
        private_key: &str,
        address_type: TonAddressType,
    ) -> Result<String, crate::Error> {
        let bytes = wallet_utils::hex_func::hex_decode(private_key)?;

        let sk = ed25519_dalek_bip32::SecretKey::from_bytes(&bytes)
            .map_err(|_e| crate::Error::PriKey("ton invalid private key".to_string()))?;

        let pk = ed25519_dalek_bip32::PublicKey::from(&sk);

        let mut sk = sk.as_bytes().to_vec();
        let pk = pk.as_bytes().to_vec();
        sk.extend(&pk);

        let key_pair = KeyPair {
            secret_key: sk,
            public_key: pk,
        };

        let version = match address_type {
            TonAddressType::V4R2 => WalletVersion::V4R2,
            TonAddressType::V5R1 => WalletVersion::V5R1,
        };

        let wallet = TonWallet::new(version, key_pair).unwrap();
        Ok(wallet.address.to_base64_url())
    }
}

//  获取派生路径
impl wallet_core::derive::GenDerivation for TonInstance {
    type Error = crate::Error;
    fn generate(
        _address_type: &Option<BtcAddressType>,
        input_index: i32,
    ) -> Result<String, crate::Error> {
        let path = if input_index < 0 {
            let i = wallet_utils::address::i32_index_to_unhardened_u32(input_index)?;
            crate::add_index(Self::TON_DERIVATION_PATH, i, true)
        } else {
            let i = input_index as u32;
            crate::add_index(Self::TON_DERIVATION_PATH, i, true)
        };
        Ok(path)
    }
}

impl wallet_core::KeyPair for TonKeyPair {
    type Error = crate::Error;

    fn network(&self) -> wallet_types::chain::network::NetworkKind {
        self.network
    }

    // 生成keypair
    fn generate_with_derivation(
        seed: Vec<u8>,
        derivation_path: &str,
        chain_code: &ChainCode,
        network: wallet_types::chain::network::NetworkKind,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let drive_path = DerivationPath::from_str(derivation_path).unwrap();

        let key = ExtendedSecretKey::from_seed(&seed)
            .unwrap()
            .derive(&drive_path)
            .unwrap();

        Ok(Self {
            tron_family: chain_code.to_owned(),
            private_key: key,
            network,
            derivation: derivation_path.to_owned(),
        })
    }

    fn private_key(&self) -> Result<String, Self::Error> {
        // Ok(self.private_key.key.to_lower_hex_string())
        Ok(self.private_key.secret_key.as_bytes().to_lower_hex_string())
    }

    fn pubkey(&self) -> String {
        self.private_key
            .public_key()
            .to_bytes()
            .to_lower_hex_string()
        // wallet_utils::hex_func::hex_encode(&self.private_key.public_key()[1..])
    }

    fn address(&self) -> String {
        let key_pair = KeyPair {
            secret_key: self.private_key.secret_key.as_bytes().to_vec(),
            public_key: self.private_key.public_key().as_bytes().to_vec(),
        };

        let wallet = TonWallet::new(WalletVersion::V4R2, key_pair).unwrap();

        let testnet = match self.network {
            wallet_types::chain::network::NetworkKind::Mainnet => false,
            _ => true,
        };
        wallet.address.to_base64_url_flags(true, testnet)
    }

    fn derivation_path(&self) -> String {
        self.derivation.clone()
    }

    fn chain_code(&self) -> ChainCode {
        self.tron_family
    }

    fn private_key_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        Ok(self.private_key.secret_key.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod test {
    use super::TonInstance;
    use crate::instance::ton::TonKeyPair;
    use tonlib_core::TonAddress;
    use wallet_core::{KeyPair, derive::GenDerivation, xpriv};
    use wallet_types::chain::chain::ChainCode;

    #[test]
    fn test_gen() {
        let phrase =
            "green pizza fix similar sentence digital pear suggest where luggage bomb because";
        let password = "";

        let xpriv = xpriv::generate_master_key(1, phrase, password).unwrap();
        let path = TonInstance::generate(&None, 0).unwrap();

        println!("path: {path}");
        let path = "m/44'/607'/0";
        let chain_code = ChainCode::Bitcoin;
        let keypair = TonKeyPair::generate_with_derivation(
            xpriv.1,
            &path,
            &chain_code,
            wallet_types::chain::network::NetworkKind::Mainnet,
        )
        .unwrap();

        println!("private key {}", keypair.private_key().unwrap());

        println!("{}", keypair.address());

        // assert_eq!(
        //     keypair.address(),
        //     "UQC1W9L_a15KdQMBQgM35W_xqTU7O-D-EIjHG8-RA6nljFVj"
        // );
    }

    #[test]
    fn test_gen1() {
        let phrase =
            "other phrase banana execute acquire scorpion amused route garage close hole barely";
        let password = "";

        let xpriv = xpriv::generate_master_key(1, phrase, password).unwrap();
        let path = TonInstance::generate(&None, 1).unwrap();

        let chain_code = ChainCode::Bitcoin;
        let keypair = TonKeyPair::generate_with_derivation(
            xpriv.1,
            &path,
            &chain_code,
            wallet_types::chain::network::NetworkKind::Mainnet,
        )
        .unwrap();

        println!("private key {}", keypair.private_key().unwrap());
        assert_eq!(
            keypair.address(),
            "UQBud2VI5S1IhaPm3OJ7wYUewhBSK7VhfPbnp_0tvvBpx7ze"
        );
    }

    #[test]
    fn test_address_format() {
        let address =
            TonAddress::from_base64_url("UQBud2VI5S1IhaPm3OJ7wYUewhBSK7VhfPbnp_0tvvBpx7ze")
                .unwrap();

        println!("可回退地址   {}", address.to_base64_url_flags(false, false));
        println!("不可回退地址 {} ", address.to_base64_url_flags(true, false));
        println!("不可回退地址 {} ", address.to_base64_std());
        println!("16进制地址 {:?} ", address.to_msg_address());
    }
}
