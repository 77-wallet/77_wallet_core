use std::str::FromStr as _;

use dogcoin::{
    PrivateKey,
    hashes::{Hash as _, HashEngine as _},
};
use ripemd160::Digest as _;
use secp256k1::{Keypair, Secp256k1};
use wallet_types::chain::{address::r#type::DogAddressType, chain, network};

#[derive(Clone)]
pub struct DogGenAddress {
    pub address_type: DogAddressType,
    pub network: network::NetworkKind,
}

impl wallet_core::address::GenAddress for DogGenAddress {
    type Address = crate::instance::Address;
    type Error = crate::Error;
    fn generate(&self, pkey: &[u8]) -> Result<Self::Address, Self::Error>
    where
        Self: Sized,
    {
        let secret_key = secp256k1::SecretKey::from_slice(pkey)?;

        let secp = dogcoin::key::Secp256k1::new();
        let keypair = secp256k1::Keypair::from_secret_key(&secp, &secret_key);

        let network = self.network;
        Ok(crate::instance::Address::DogAddress(
            generate_address_with_xpriv(&self.address_type, &secp, keypair, network)?,
        ))
    }

    fn chain_code(&self) -> &chain::ChainCode {
        &chain::ChainCode::Bitcoin
    }
}

fn ripemd160_sha256(public_key: &[u8]) -> Vec<u8> {
    let mut hasher = ripemd160::Ripemd160::new();

    let hash_of_bytes = dogcoin::hashes::sha256::Hash::hash(public_key);
    // process input message
    hasher.update(hash_of_bytes);
    hasher.finalize().to_vec()
}

fn sha256_twice(raw: &[u8]) -> Vec<u8> {
    let hash = &dogcoin::hashes::sha256::Hash::hash(raw).hash_again();
    hash.to_byte_array()[..4].to_vec()
}

fn private_key(seed: Vec<u8>) -> Result<bitcoin::bip32::Xpriv, crate::Error> {
    Ok(bitcoin::bip32::Xpriv::new_master(
        bitcoin::Network::Bitcoin,
        &seed,
    )?)
}

fn generate_xpriv(
    seed: Vec<u8>,
    path: &str,
    secp: &secp256k1::Secp256k1<bitcoin::secp256k1::All>,
) -> Result<bitcoin::bip32::Xpriv, crate::Error> {
    let xpiri: bitcoin::bip32::Xpriv = private_key(seed)?;
    let path = bitcoin::bip32::DerivationPath::from_str(path)?;

    Ok(xpiri.derive_priv(secp, &path)?)
}

pub(crate) fn generate_address(
    address_type: &DogAddressType,
    seed: &[u8],
    derivation_path: &str,
    network: network::NetworkKind,
) -> Result<String, crate::Error> {
    let secp = Secp256k1::new();
    let xpriv = generate_xpriv(seed.to_vec(), derivation_path, &secp)?;
    let keypair = xpriv.to_keypair(&secp);
    generate_address_with_xpriv(address_type, &secp, keypair, network)
}

pub fn generate_address_by_seckey(
    address_type: &DogAddressType,
    network: network::NetworkKind,
    seckey: String,
) -> Result<String, crate::Error> {
    let secp = Secp256k1::new();

    let pk = PrivateKey::from_wif(&seckey).map_err(|e| crate::Error::PriKey(e.to_string()))?;
    let b = pk.to_bytes();

    let keypair = Keypair::from_seckey_slice(&secp, b.as_ref())
        .map_err(|e| crate::Error::PriKey(e.to_string()))?;

    generate_address_with_xpriv(&address_type, &secp, keypair, network)
}

pub fn generate_address_with_xpriv(
    address_type: &DogAddressType,
    secp: &Secp256k1<secp256k1::All>,
    // xpriv: dogcoin::bip32::Xpriv,
    keypair: secp256k1::Keypair,
    network: network::NetworkKind,
) -> Result<String, crate::Error> {
    match address_type {
        DogAddressType::P2pkh => legacy(keypair, network),
        // DogAddressType::P2sh => todo!(),
        // DogAddressType::P2shWpkh => todo!(),
        DogAddressType::P2shWpkh => p2sh_p2wpkh_address(keypair, network),
        DogAddressType::P2wpkh => p2wpkh_address(keypair, network),
        // DogAddressType::P2wsh => todo!(),
        DogAddressType::P2tr => p2tr_address(keypair, secp, network),
        // DogAddressType::P2trSh => todo!(),
        _ => Err(crate::Error::DogAddressTypeCantGenDerivationPath),
    }
}

pub(crate) fn legacy(
    // xpriv: dogcoin::bip32::Xpriv,
    keypair: secp256k1::Keypair,
    // secp: &Secp256k1<secp256k1::All>,
    network: network::NetworkKind,
) -> Result<String, crate::Error> {
    // let keypair = xpriv.to_keypair(secp);
    let pubkey = keypair.public_key().serialize();
    let res = generate_p2pkh_address(&pubkey, network)?;
    Ok(res)
}

// 辅助函数：根据网络类型获取 Bech32 的 HRP
fn get_bech32_hrp(network: network::NetworkKind) -> Result<bech32::Hrp, crate::Error> {
    match network {
        network::NetworkKind::Mainnet => Ok(wallet_utils::parse_func::parse_bech32_hrp("dog1")?),
        network::NetworkKind::Testnet => Ok(wallet_utils::parse_func::parse_bech32_hrp("tn1")?),
        network::NetworkKind::Regtest => Ok(wallet_utils::parse_func::parse_bech32_hrp("bcrt")?),
    }
}

/// 获取 P2SH 的版本前缀根据网络类型
fn get_p2sh_version(network: network::NetworkKind) -> u8 {
    match network {
        network::NetworkKind::Mainnet => 0x16, // P2SH mainnet
        network::NetworkKind::Testnet => 0xC4, // P2SH testnet/regtest
        network::NetworkKind::Regtest => 0xC4, // P2SH testnet/regtest
    }
}

/// 获取 P2PKH 的版本前缀根据网络类型
fn get_p2pkh_version(network: network::NetworkKind) -> u8 {
    match network {
        network::NetworkKind::Mainnet => 0x1E, // P2PKH mainnet
        network::NetworkKind::Testnet => 0x71, // P2PKH testnet/regtest
        network::NetworkKind::Regtest => 0x6F, // P2PKH testnet/regtest
    }
}

// 隔离见证（原生）
pub(crate) fn p2wpkh_address(
    keypair: secp256k1::Keypair,
    // xpriv: dogcoin::bip32::Xpriv,
    // secp: &Secp256k1<secp256k1::All>,
    network: network::NetworkKind,
) -> Result<String, crate::Error> {
    // let keypair = xpriv.to_keypair(&secp);
    let pubkey = keypair.public_key().serialize();
    let res = generate_p2wpkh_address(&pubkey, network)?;

    Ok(res)
}

// Taproot
pub(crate) fn p2tr_address(
    keypair: secp256k1::Keypair,
    // xpriv: dogcoin::bip32::Xpriv,
    secp: &Secp256k1<secp256k1::All>,
    network: network::NetworkKind,
) -> Result<String, crate::Error> {
    // let keypair = xpriv.to_keypair(&secp);
    let res = generate_p2tr_address(&keypair, secp, network)?;

    Ok(res)
}

// 隔离见证（兼容）
pub(crate) fn p2sh_p2wpkh_address(
    keypair: secp256k1::Keypair,
    // xpriv: dogcoin::bip32::Xpriv,
    // secp: &Secp256k1<secp256k1::All>,
    network: network::NetworkKind,
) -> Result<String, crate::Error> {
    // let keypair = xpriv.to_keypair(&secp);
    let pubkey = keypair.public_key().serialize();

    let res = generate_p2sh_p2wpkh_address(&pubkey, network);

    Ok(res)
}

// 1. P2PKH 地址
pub(crate) fn generate_p2pkh_address(
    public_key: &[u8],
    network: network::NetworkKind,
) -> Result<String, crate::Error> {
    // 验证公钥长度
    if !(public_key.len() == 33 || public_key.len() == 65) {
        return Err(crate::Error::InvalidPublicKeyLength);
    }

    // Step 1: 生成公钥哈希 (RIPEMD-160(SHA-256(public_key)))
    // acquire hash digest in the form of GenericArray,
    // which in this case is equivalent to [u8; 20]
    let pubkey_hash = ripemd160_sha256(public_key);

    // Step 2: 添加版本前缀 (0x00 for mainnet, 0x6F for testnet)
    // Step 2: 添加版本前缀根据网络类型
    let version = get_p2pkh_version(network);

    let mut versioned_payload = Vec::with_capacity(1 + pubkey_hash.len());
    versioned_payload.push(version);
    versioned_payload.extend_from_slice(&pubkey_hash);

    // Step 3: 计算校验和 (SHA-256(SHA-256(versioned_payload)))[..4]
    // let a = &dogcoin::hashes::sha256::Hash::hash(&versioned_payload);
    // let checksum = dogcoin::hashes::sha256::Hash::hash(a.as_byte_array())[..4].to_vec();
    let checksum = sha256_twice(&versioned_payload);

    // Step 4: 生成最终地址 (Base58Check编码)
    let address_bytes = [versioned_payload, checksum].concat();
    Ok(bs58::encode(address_bytes).into_string())
}

// 3. P2SH-P2WPKH 地址 隔离见证（兼容）
pub(crate) fn generate_p2sh_p2wpkh_address(
    witness_program: &[u8],
    network: network::NetworkKind,
) -> String {
    let witness_program = ripemd160_sha256(witness_program);

    // Step 1: 创建嵌套脚本 (0x0014 + 公钥哈希)
    let script = [b"\x00\x14", witness_program.as_slice()].concat();

    // Step 2: 计算脚本哈希 (RIPEMD-160(SHA-256(script)))

    // let script_hash = ripemd160(&sha256(&script));
    let script_hash = ripemd160_sha256(&script);

    // Step 3: 添加版本前缀 (0x05 for mainnet, 0xC4 for testnet)
    let version = get_p2sh_version(network);
    let mut versioned_payload = Vec::with_capacity(1 + 20);
    versioned_payload.push(version);
    versioned_payload.extend_from_slice(&script_hash);

    // Step 4: 计算校验和 (SHA-256(SHA-256(versioned_payload)))[..4]
    // let checksum = sha256(&sha256(&versioned_payload))[..4].to_vec();
    let checksum = sha256_twice(&versioned_payload);

    // Step 5: 生成最终地址 (Base58Check编码)
    let address_bytes = [versioned_payload, checksum].concat();
    bs58::encode(address_bytes).into_string()
}

// 5. P2WPKH 地址 隔离见证（原生）
pub(crate) fn generate_p2wpkh_address(
    public_key: &[u8],
    network: network::NetworkKind,
) -> Result<String, crate::Error> {
    // 验证公钥长度
    if !(public_key.len() == 33 || public_key.len() == 65) {
        return Err(crate::Error::InvalidPublicKeyLength);
    }

    // Step 1: 生成公钥哈希 (RIPEMD-160(SHA-256(public_key)))
    let pubkey_hash = ripemd160_sha256(public_key);

    // Step 2: 创建见证程序 (0x00 + 公钥哈希)
    // 对于 P2WPKH，见证程序是公钥哈希，见证版本为 0
    let witness_program = pubkey_hash.as_slice();

    // Step 3: 获取 HRP
    let hrp = get_bech32_hrp(network)?;

    // Step 4: 编码地址
    Ok(bech32::segwit::encode(
        hrp,
        bech32::segwit::VERSION_0,
        witness_program,
    )?)
}

// // 7.p2tr
pub(crate) fn generate_p2tr_address(
    keypair: &secp256k1::Keypair,
    secp: &Secp256k1<dogcoin::secp256k1::All>,
    network: network::NetworkKind,
) -> Result<String, crate::Error> {
    // Step 1: 提取 x-only 公钥
    let (xonly_pubkey, _) = keypair.x_only_public_key();

    // let (output_key,_) = xonly_pubkey.tap_tweak(&secp, None);
    // let pubkey = output_key.to_inner().serialize();
    // println!("pubkey: {pubkey:?}");

    // Step 2: 调整 x-only 公钥 (使用 SHA256(xonly_pubkey) 作为 tweak)
    // let mut engine = secp256k1::hashes::sha256::HashEngine::default();
    let mut engine = secp256k1::hashes::sha256::HashEngine::from_midstate(
        secp256k1::hashes::sha256::Midstate::hash_tag("TapTweak".as_bytes()),
        64,
    );

    // Step 2: 计算 Taproot tweak
    let tweak = dogcoin::TapTweakHash::from_key_and_tweak(xonly_pubkey, None).to_scalar();

    // let mut engine = dogcoin::TapTweakHash::engine();
    // let midstate = engine.midstate();
    // println!("midstate: {:?}", midstate);

    engine.input(&xonly_pubkey.serialize());

    let hash1 = &secp256k1::hashes::sha256::Hash::from_engine(engine);
    // let checksum = hash1.into_inner().as_slice()[..4].to_vec().as_slice();
    let checksum = hash1.to_byte_array();

    let _tweak = secp256k1::Scalar::from_be_bytes(checksum)?;

    // let hash = &dogcoin::hashes::sha256::Hash::hash(xonly_pubkey.to_string().as_bytes());
    // let tweak = hash.to_byte_array();

    // let tweak = dogcoin::secp256k1::Scalar::from_be_bytes(tweak).unwrap();
    // let (mut public_key, _) = keypair.x_only_public_key();
    // let original = public_key;
    let (tweaked_xonly_pubkey, parity) = xonly_pubkey.add_tweak(secp, &tweak)?;
    debug_assert!(xonly_pubkey.tweak_add_check(secp, &tweaked_xonly_pubkey, parity, tweak));
    let pubkey = tweaked_xonly_pubkey.serialize();

    // let b = xonly_pubkey.tweak_add_check(&secp, &tweaked_xonly_pubkey, parity, tweak);
    // println!("ok: {b}");
    // let tweak =  Sha256::digest(&xonly_pubkey);
    // let tweaked_xonly_pubkey = tweak_xonly_pubkey(&xonly_pubkey, &tweak);

    // Step 3: 计算 Taproot 哈希 (SHA-256(tweaked_xonly_pubkey))
    // let taproot_hash =
    //     // &dogcoin::hashes::sha256::Hash::hash(tweaked_xonly_pubkey.to_string().as_bytes())
    //     &dogcoin::hashes::sha256::Hash::hash(&pubkey)
    //         .to_byte_array();
    // let taproot_hash = Sha256::digest(&tweaked_xonly_pubkey);

    // Step 4: 创建见证程序 (0x01 + Taproot 哈希)
    // let witness_program = [
    //     vec![dogcoin::WitnessVersion::V1 as u8],
    //     taproot_hash.to_vec(),
    // ]
    // .concat();

    // Step 6: 获取 HRP
    let hrp = get_bech32_hrp(network)?;

    let witness_program = pubkey.to_vec();
    // Step 7: 使用 Bech32m 编码生成最终地址
    let taproot_address = bech32::segwit::encode(hrp, bech32::segwit::VERSION_1, &witness_program)?;
    // bech32::encode("bc", witness_program.to_base32(), Variant::Bech32m).unwrap()
    Ok(taproot_address)
}
