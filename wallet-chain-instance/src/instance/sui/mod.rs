pub mod address;
use solana_sdk::signer::Signer;
use wallet_core::{derive::Derive, KeyPair};
use wallet_types::chain::{address::r#type::BtcAddressType, chain::ChainCode};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct SuiInstance {
    pub(crate) chain_code: ChainCode,
    pub network: wallet_types::chain::network::NetworkKind,
}

impl wallet_core::derive::GenDerivation for SuiInstance {
    type Error = crate::Error;
    fn generate(
        _address_type: &Option<BtcAddressType>,
        input_index: i32,
    ) -> Result<String, crate::Error> {
        let index = wallet_utils::address::i32_index_to_unhardened_u32(input_index)?;
        let path = crate::add_solana_index(wallet_types::constant::SUI_DERIVATION_PATH, index);
        Ok(path)
    }
}

impl Derive for SuiInstance {
    type Error = crate::Error;
    type Item = SuiKeyPair;

    fn derive_with_derivation_path(
        &self,
        seed: Vec<u8>,
        derivation_path: &str,
    ) -> Result<Self::Item, Self::Error> {
        SuiKeyPair::generate_with_derivation(seed, derivation_path, &self.chain_code, self.network)
    }
}

pub struct SuiKeyPair {
    sui_family: ChainCode,
    keypair: solana_sdk::signature::Keypair,
    pubkey: String,
    derivation: String,
    network: wallet_types::chain::network::NetworkKind,
}

impl KeyPair for SuiKeyPair {
    type Error = crate::Error;

    fn generate_with_derivation(
        seed: Vec<u8>,
        derivation_path: &str,
        chain_code: &ChainCode,
        network: wallet_types::chain::network::NetworkKind,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let derivation =
      sui_crypto::
            // solana_sdk::derivation_path::DerivationPath::from_absolute_path_str(derivation_path)
            //     .map_err(|e| crate::Error::Keypair(crate::KeypairError::Solana(e.to_string())))?;
        let keypair =
            solana_sdk::signature::keypair_from_seed_and_derivation_path(&seed, Some(derivation))
                .map_err(|e| crate::Error::Keypair(crate::KeypairError::Solana(e.to_string())))?;

        let pubkey = keypair.pubkey().to_string();

        Ok(Self {
            sui_family: chain_code.to_owned(),
            pubkey,
            keypair,
            derivation: derivation_path.to_string(),
            network,
        })
    }

    fn network(&self) -> wallet_types::chain::network::NetworkKind {
        self.network
    }

    fn private_key(&self) -> Result<String, Self::Error> {
        // solana_sdk::derivation_path::DerivationPath
        Ok(self.keypair.to_base58_string())
    }
    fn pubkey(&self) -> String {
        self.pubkey.clone()
    }

    fn address(&self) -> String {
        self.keypair.pubkey().to_string()
    }

    fn derivation_path(&self) -> String {
        self.derivation.clone()
    }

    fn chain_code(&self) -> wallet_types::chain::chain::ChainCode {
        self.sui_family
    }

    fn private_key_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        Ok(self.keypair.to_bytes().to_vec())
        // Ok().map_err(|e| crate::Error::Parse(e.into()))?)
    }
}

#[cfg(test)]
mod test {
    use coins_bip32::xkeys::XPriv;
    use hex::encode;
    use secp256k1::PublicKey;
    use sha3::{Digest, Keccak256};
    // use wallet_core::KeyPair;

    // use super::TronKeyPair;

    #[test]
    fn test_trx() {
        let seed = "5b56c417303faa3fcba7e57400e120a0ca83ec5a4fc9ffba757fbe63fbd77a89a1a3be4c67196f57c39a88b76373733891bfaba16ed27a813ceed498804c0570";
        let _seed = hex::decode(seed).unwrap();

        // let pri_key = XPriv::root_from_seed(seed.as_slice(), None).unwrap();
        // // let keypair = TronKeyPair::generate(pri_key).unwrap();
        // let keypair = TronKeyPair::generate(seed, 1).unwrap();
    }

    #[test]
    fn test_gen_pk() {
        let seed = "8200bcbcffbe52e9e720510dd0ec67dccdb14856b6443527f3492f5a28275d5799b0fec07aa11b3dca906c4552b161f116af4debb386696e8fe939a363706f6c";
        let secp = secp256k1::Secp256k1::new();
        let seed = hex::decode(seed).unwrap();

        let pri_key = XPriv::root_from_seed(seed.as_slice(), None).unwrap();

        let derive_key = pri_key.derive_path("m/44'/784'/0'/0'/0'").unwrap();

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
