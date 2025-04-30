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
        todo!()
        //     let derivation =
        //   sui_crypto::
        //         // solana_sdk::derivation_path::DerivationPath::from_absolute_path_str(derivation_path)
        //         //     .map_err(|e| crate::Error::Keypair(crate::KeypairError::Solana(e.to_string())))?;
        //     let keypair =
        //         solana_sdk::signature::keypair_from_seed_and_derivation_path(&seed, Some(derivation))
        //             .map_err(|e| crate::Error::Keypair(crate::KeypairError::Solana(e.to_string())))?;

        //     let pubkey = keypair.pubkey().to_string();

        //     Ok(Self {
        //         sui_family: chain_code.to_owned(),
        //         pubkey,
        //         keypair,
        //         derivation: derivation_path.to_string(),
        //         network,
        //     })
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

use coins_bip39::Mnemonic;
use ed25519_dalek::SecretKey;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha512};
use sha3::Sha3_256;
type HmacSha512 = Hmac<Sha512>;

// 核心派生逻辑 (完全匹配 Sui TypeScript SDK)
fn derive_ed25519_from_mnemonic(mnemonic: &str, path: &str) -> [u8; 32] {
    // 1. 生成 BIP39 种子
    let mnemonic = Mnemonic::<coins_bip39::English>::new_from_phrase(mnemonic).unwrap();
    let seed = mnemonic.to_seed(None).unwrap().to_vec();

    // let seed = Seed::new(&Mnemonic::from_phrase(mnemonic).unwrap(), "");
    // let seed_bytes = seed.as_bytes();

    // 2. 解析派生路径 (例如 m/44'/784'/0'/0'/0')
    let segments: Vec<&str> = path.split('/').skip(1).collect();
    let mut key = hmac_sha512(b"ed25519 seed", seed.as_slice());

    for segment in segments {
        let mut data = [0u8; 4 + 32];
        data[..32].copy_from_slice(&key[..32]);
        let index = parse_segment(segment);
        data[32..].copy_from_slice(&index.to_be_bytes());
        key = hmac_sha512(&key[32..], &data);
    }

    key[..32].try_into().unwrap()
}

// HMAC-SHA512 辅助函数
fn hmac_sha512(key: &[u8], data: &[u8]) -> [u8; 64] {
    let mut hmac = HmacSha512::new_from_slice(key).unwrap();
    hmac.update(data);
    hmac.finalize().into_bytes().into()
}

// 解析派生路径段 (支持硬化)
fn parse_segment(segment: &str) -> u32 {
    let hardened = segment.ends_with('\'');
    let index: u32 = segment.trim_end_matches('\'').parse().unwrap();
    index | if hardened { 0x80000000 } else { 0 }
}

// 生成 Sui 地址
fn to_sui_address(pub_key: &[u8; 32]) -> String {
    let mut hasher_input = vec![0x00]; // Ed25519 标识
    hasher_input.extend_from_slice(pub_key);

    let hash = Sha3_256::digest(&hasher_input);

    let mut address = vec![0x00];
    address.extend_from_slice(&hash[..20]);

    let address = bs58::encode(address).into_string();
    address
}

#[cfg(test)]
mod test {
    use coins_bip32::xkeys::XPriv;
    use ed25519_dalek::SecretKey;
    use hex::encode;
    use secp256k1::PublicKey;
    use sha3::{Digest, Keccak256};

    use crate::instance::sui::{derive_ed25519_from_mnemonic, to_sui_address};

    // use wallet_core::KeyPair;

    // use super::TronKeyPair;

    // #[test]
    // fn test_sui_address() {
    //     let seed = "5b56c417303faa3fcba7e57400e120a0ca83ec5a4fc9ffba757fbe63fbd77a89a1a3be4c67196f57c39a88b76373733891bfaba16ed27a813ceed498804c0570";
    //     // let seed = "aa62ba53baee195f989cfb93af4d409d006f4970c8274191152711ce597d1d779f586b56e2fa378e3d600cfa215b5aeda9607e2ddc30707294bdb2bd656bd166";
    //     let seed = hex::decode(seed).unwrap();
    //     println!("len: {}", seed.len());
    //     // let seed: [u8; 32] = [
    //     //     0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
    //     //     0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78,
    //     //     0x9a, 0xbc, 0xde, 0xf0,
    //     // ];

    //     let address = generate_sui_address(&seed);
    //     println!("address: {}", address);
    // }

    #[test]
    fn test_sui_address() {
        let test_cases = vec![
        (
            "film crazy soon outside stand loop subway crumble thrive popular green nuclear struggle pistol arm wife phrase warfare march wheat nephew ask sunny firm",
            "m/44'/784'/0'/0'/0'",
            "0xa2d14fad60c56049ecf75246a481934691214ce413e6a8ae2fe6834c173a6133",
        ),
        // 其他测试用例...
    ];

        for (mnemonic, path, expected_addr) in test_cases {
            let seed = derive_ed25519_from_mnemonic(mnemonic, path);
            let key_pair = ed25519_dalek::Keypair::from_bytes(&seed).unwrap();

            // let signing_key = ed25519_dalek::Keypair::from(&secret_key);

            // let pub_key = signing_key.verifying_key().to_bytes();

            let addr = to_sui_address(&key_pair.public.to_bytes());
            assert_eq!(addr, expected_addr.trim_start_matches("0x"));
        }
    }

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
