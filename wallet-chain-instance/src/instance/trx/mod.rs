pub mod address;
use anychain_core::{Address, PublicKey as _};
use anychain_tron::{TronAddress, TronFormat};
use coins_bip32::xkeys::XPriv;
use wallet_core::{derive::Derive, KeyPair};
use wallet_types::chain::{address::r#type::BtcAddressType, chain::ChainCode};

pub struct TronKeyPair {
    tron_family: ChainCode,
    private_key: libsecp256k1::SecretKey,
    pubkey: String,
    address: TronAddress,
    derivation: String,
    network: wallet_types::chain::network::NetworkKind,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct TronInstance {
    pub(crate) chain_code: ChainCode,
    pub network: wallet_types::chain::network::NetworkKind,
}

impl wallet_core::derive::GenDerivation for TronInstance {
    type Error = crate::Error;
    fn generate(
        _address_type: &Option<BtcAddressType>,
        input_index: i32,
    ) -> Result<String, crate::Error> {
        let path = if input_index < 0 {
            let i = wallet_utils::address::i32_index_to_unhardened_u32(input_index)?;
            crate::add_index(wallet_types::constant::TRON_HARD_DERIVATION_PATH, i, true)
        } else {
            let i = input_index as u32;
            crate::add_index(wallet_types::constant::TRON_DERIVATION_PATH, i, false)
        };
        Ok(path)
    }
}

impl Derive for TronInstance {
    type Error = crate::Error;
    type Item = TronKeyPair;

    fn derive_with_derivation_path(
        &self,
        seed: Vec<u8>,
        derivation_path: &str,
    ) -> Result<Self::Item, Self::Error> {
        TronKeyPair::generate_with_derivation(seed, derivation_path, &self.chain_code, self.network)
    }
}

impl wallet_core::KeyPair for TronKeyPair {
    type Error = crate::Error;

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
        let private_key = signingkey.to_bytes();

        let private_key = libsecp256k1::SecretKey::parse_slice(&private_key)
            .map_err(|e| crate::Error::Keypair(e.into()))?;
        let pubkey = anychain_tron::TronPublicKey::from_secret_key(&private_key);

        let address = TronAddress::from_secret_key(&private_key, &TronFormat::Standard).unwrap();
        // let address = alloy::signers::utils::secret_key_to_address(signingkey);
        Ok(Self {
            tron_family: chain_code.to_owned(),
            private_key,
            pubkey: pubkey.to_string(),
            address,
            derivation: derivation_path.to_string(),
            network,
        })
    }

    fn private_key(&self) -> Result<String, Self::Error> {
        let res = self.private_key.serialize();
        Ok(hex::encode(res))
    }

    fn pubkey(&self) -> String {
        self.pubkey.clone()
    }

    fn address(&self) -> String {
        self.address.to_base58()
    }

    fn derivation_path(&self) -> String {
        self.derivation.clone()
    }

    fn chain_code(&self) -> ChainCode {
        self.tron_family
    }

    fn private_key_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        Ok(self.private_key.serialize().to_vec())
        // Ok(hex::decode(self.private_key()?).map_err(|e| crate::Error::Parse(e.into()))?)
    }
}

pub fn secret_key_to_address(
    signingkey: &coins_bip32::ecdsa::SigningKey,
) -> Result<TronAddress, crate::Error> {
    let private_key = signingkey.to_bytes();

    let private_key = libsecp256k1::SecretKey::parse_slice(&private_key)
        .map_err(|e| crate::Error::Keypair(e.into()))?;
    let address = TronAddress::from_secret_key(&private_key, &TronFormat::Standard).unwrap();
    Ok(address)
}

#[cfg(test)]
mod test {
    use coins_bip32::xkeys::XPriv;
    use hex::encode;
    use secp256k1::PublicKey;
    use sha3::{Digest, Keccak256};

    #[test]
    fn test_trx() {
        let seed = "5b56c417303faa3fcba7e57400e120a0ca83ec5a4fc9ffba757fbe63fbd77a89a1a3be4c67196f57c39a88b76373733891bfaba16ed27a813ceed498804c0570";
        let _seed = hex::decode(seed).unwrap();
    }

    #[test]
    fn test_gen_pk() {
        let seed = "5b56c417303faa3fcba7e57400e120a0ca83ec5a4fc9ffba757fbe63fbd77a89a1a3be4c67196f57c39a88b76373733891bfaba16ed27a813ceed498804c0570";
        let secp = secp256k1::Secp256k1::new();
        let seed = hex::decode(seed).unwrap();

        let pri_key = XPriv::root_from_seed(seed.as_slice(), None).unwrap();

        let derive_key = pri_key.derive_path("m/44'/195'/0'/0/0").unwrap();

        let signingkey: &coins_bip32::ecdsa::SigningKey = derive_key.as_ref();
        let private_key = signingkey.to_bytes();

        // let key: &coins_bip32::prelude::SigningKey = master_key.as_ref();
        let key = alloy::hex::encode(private_key);

        tracing::info!("master key: {:?}", key);

        let secret_key = secp256k1::SecretKey::from_slice(&private_key).unwrap();

        // Step 2: 从私钥生成公钥
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let serialized_public_key = public_key.serialize_uncompressed();

        // Step 3: 计算公钥的Keccak256哈希值
        let mut hasher = Keccak256::new();
        hasher.update(&serialized_public_key[1..]);
        let result = hasher.finalize();

        // Step 4: 取哈希值的后20字节
        let address_bytes = &result[12..];

        // Step 5: TRON地址前缀为41，拼接前缀
        let mut tron_address = vec![0x41];
        tron_address.extend_from_slice(address_bytes);

        // 将地址格式化为十六进制字符串
        let tron_address_hex = encode(tron_address);

        // 输出私钥和TRON地址
        println!("Private Key: {}", encode(secret_key.as_ref()));
        println!("TRON Address: {}", tron_address_hex);
    }
}
