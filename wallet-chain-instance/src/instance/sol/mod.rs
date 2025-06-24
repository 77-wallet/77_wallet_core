pub mod address;
use solana_sdk::signer::Signer;
use wallet_core::{KeyPair, derive::Derive};
use wallet_types::chain::{address::r#type::BtcAddressType, chain::ChainCode};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct SolanaInstance {
    pub(crate) chain_code: ChainCode,
    pub network: wallet_types::chain::network::NetworkKind,
}

impl wallet_core::derive::GenDerivation for SolanaInstance {
    type Error = crate::Error;
    fn generate(
        _address_type: &Option<BtcAddressType>,
        input_index: i32,
    ) -> Result<String, crate::Error> {
        let index = wallet_utils::address::i32_index_to_unhardened_u32(input_index)?;
        let path = crate::add_solana_index(wallet_types::constant::SOLANA_DERIVATION_PATH, index);
        Ok(path)
    }
}

impl Derive for SolanaInstance {
    type Error = crate::Error;
    type Item = SolanaKeyPair;

    fn derive_with_derivation_path(
        &self,
        seed: Vec<u8>,
        derivation_path: &str,
    ) -> Result<Self::Item, Self::Error> {
        SolanaKeyPair::generate_with_derivation(
            seed,
            derivation_path,
            &self.chain_code,
            self.network,
        )
    }
}

pub struct SolanaKeyPair {
    solana_family: ChainCode,
    keypair: solana_sdk::signature::Keypair,
    pubkey: String,
    derivation: String,
    network: wallet_types::chain::network::NetworkKind,
}

impl KeyPair for SolanaKeyPair {
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
            solana_sdk::derivation_path::DerivationPath::from_absolute_path_str(derivation_path)
                .map_err(|e| crate::Error::Keypair(crate::KeypairError::Solana(e.to_string())))?;
        let keypair =
            solana_sdk::signature::keypair_from_seed_and_derivation_path(&seed, Some(derivation))
                .map_err(|e| crate::Error::Keypair(crate::KeypairError::Solana(e.to_string())))?;

        let pubkey = keypair.pubkey().to_string();

        Ok(Self {
            solana_family: chain_code.to_owned(),
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
        self.solana_family
    }

    fn private_key_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        Ok(self.keypair.to_bytes().to_vec())
        // Ok().map_err(|e| crate::Error::Parse(e.into()))?)
    }
}

pub fn secret_key_to_address(pkey: &[u8]) -> Result<solana_sdk::pubkey::Pubkey, crate::Error> {
    let keypair = solana_sdk::signer::keypair::Keypair::from_bytes(pkey).unwrap();
    Ok(keypair.pubkey())
}

pub fn address_from_secret_key(prik: &str) -> Result<solana_sdk::pubkey::Pubkey, crate::Error> {
    let keypair = solana_sdk::signer::keypair::Keypair::from_base58_string(prik);
    Ok(keypair.pubkey())
}

#[cfg(test)]
mod tests {
    use coins_bip39::{English, Mnemonic};

    const SOL_COIN_TYPE: u32 = 501 | 0x80000000; // 784' 的硬化编码
    const BIP44_PURPOSE: u32 = 44 | 0x80000000; // 44' 的硬化编码

    #[test]
    fn test_i32_as_u32() {
        // 测试正数转换
        let positive: i32 = 42;
        let positive_converted = positive as u32;
        assert_eq!(
            positive_converted, 42,
            "Positive i32 should convert to the same u32 value"
        );
        let negative: i32 = -4669;

        let index = if negative < 0 {
            negative
                .checked_add_unsigned(coins_bip32::BIP32_HARDEN)
                .unwrap() as u32
        } else {
            negative as u32
        };

        println!("index: {}", index);

        // 测试负数转换
        let negative_converted = negative as u32;
        // -42 的二进制表示转换为 u32 会得到一个较大的无符号整数
        let res = u32::MAX - 4668;
        println!("res: {}", res);
        assert_eq!(
            negative_converted, res,
            "Negative i32 should convert to a large u32 value"
        );
    }

    #[test]
    fn test_official_vector() {
        let mnemonic =
        // "";
        "film crazy soon outside stand loop subway crumble thrive popular green nuclear struggle pistol arm wife phrase warfare march wheat nephew ask sunny firm";
        // 1. 生成 BIP-39 种子（空密码）
        let mnemonic =
            Mnemonic::<English>::new_from_phrase(mnemonic).expect("Invalid mnemonic phrase");
        let seed = mnemonic.to_seed(Some("")).unwrap(); // 注意：必须使用 Some("")

        // 2. 构造完整派生路径
        let path = format!(
            "m/{}'/{}'/2147483647'/0",   // 官方测试用例路径
            BIP44_PURPOSE & !0x80000000, // 显示逻辑值 44'
            SOL_COIN_TYPE & !0x80000000  // 显示逻辑值 501'
        );
        println!("path: {path}");
        let derivation = solana_sdk::derivation_path::DerivationPath::from_absolute_path_str(&path)
            .map_err(|e| crate::Error::Keypair(crate::KeypairError::Solana(e.to_string())))
            .unwrap();
        let keypair =
            solana_sdk::signature::keypair_from_seed_and_derivation_path(&seed, Some(derivation))
                .map_err(|e| crate::Error::Keypair(crate::KeypairError::Solana(e.to_string())))
                .unwrap();

        use solana_sdk::signature::Signer as _;

        let pubkey = keypair.pubkey().to_string();
        println!("address: {}", pubkey);
        // "suiprivkey1qr4w9sqf2dlq9uwpml6gtyr9mwhwlgyc40nnpf8uk5k9yuzt0q29vep62tu";
        // assert_eq!(
        //     address,
        //     // "0xa2d14fad60c56049ecf75246a481934691214ce413e6a8ae2fe6834c173a6133"
        //     "0x885f29a4f1b4d63822728a1b1811d0278c4e25f27d3754ddd387cd34f9482d0f"
        // );
    }
}
