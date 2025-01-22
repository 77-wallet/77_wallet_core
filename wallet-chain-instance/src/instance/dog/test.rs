#![allow(unused)]
use bitcoin::{
    bip32::{DerivationPath, Xpriv},
    Network,
};
use litecoin::{
    hashes::Hash as _,
    key::{Keypair, Secp256k1},
};
use ripemd160::Digest as _;
use secp256k1::hashes::HashEngine as _;
use std::str::FromStr;

fn private_key() -> Xpriv {
    // let mnemonic = "victory member rely dirt treat woman boring tomato two hollow erosion drop";
    let mnemonic =
        "chuckle practice chicken permit swarm giant improve absurd melt kitchen oppose scrub";
    let mnemonic =
        coins_bip39::Mnemonic::<coins_bip39::English>::new_from_phrase(mnemonic).unwrap();
    // 生成种子
    // let seed = mnemonic.to_seed(Some("12345678")).unwrap();
    let seed = mnemonic.to_seed(Some("")).unwrap();
    let xpriv = Xpriv::new_master(Network::Bitcoin, &seed).unwrap();
    let pkey = xpriv.private_key.secret_bytes();
    tracing::warn!("pkey: {pkey:?}");

    tracing::warn!("xprive: {:?}", xpriv.encode());
    xpriv
}

fn derivation_path(path: &str) -> Result<DerivationPath, crate::Error> {
    // DerivationPath::from_str("m/86'/0'/0'/0/0").expect("Invalid derivation path");
    Ok(DerivationPath::from_str(path)?)
}

fn derivate_key(
    secp: &Secp256k1<bitcoin::secp256k1::All>,
    path: &str,
) -> Result<Xpriv, crate::Error> {
    let xpiri = private_key();
    let path = derivation_path(path)?;

    Ok(xpiri.derive_priv(secp, &path)?)
}

fn sha256(raw: &[u8]) -> Vec<u8> {
    let hash = &litecoin::hashes::sha256::Hash::hash(raw);
    hash.to_byte_array()[..4].to_vec()
}

// // 2. P2SH 地址
// fn generate_p2sh_address(script_hash: &[u8], is_mainnet: bool) -> String {
//     // Step 1: 获取脚本哈希
//     // (这里假设 script_hash 已经是脚本的 RIPEMD-160(SHA-256) 哈希)

//     // Step 2: 添加版本前缀 (0x05 for mainnet, 0xC4 for testnet)
//     let version = if is_mainnet { b"\x05" } else { b"\xC4" };
//     let versioned_payload = [version, script_hash].concat();

//     // Step 3: 计算校验和 (SHA-256(SHA-256(versioned_payload)))[..4]
//     let checksum = sha256_twice(&versioned_payload);

//     // Step 4: 生成最终地址 (Base58Check编码)
//     let address_bytes = [versioned_payload, checksum].concat();
//     bs58::encode(address_bytes).into_string()
// }

// // 4. P2SH-P2WSH 地址
// fn generate_p2sh_p2wsh_address(witness_script: &[u8], is_mainnet: bool) -> String {
//     // Step 1: 创建嵌套脚本 (0x0020 + 脚本哈希)
//     let script = [b"\x00\x20", &sha256(witness_script)].concat();

//     // Step 2: 计算脚本哈希 (RIPEMD-160(SHA-256(script)))
//     let script_hash = ripemd160(&sha256(&script));

//     // Step 3: 添加版本前缀 (0x05 for mainnet, 0xC4 for testnet)
//     let version = if is_mainnet { b"\x05" } else { b"\xC4" };
//     let versioned_payload = [version, &script_hash].concat();

//     // Step 4: 计算校验和 (SHA-256(SHA-256(versioned_payload)))[..4]
//     let checksum = sha256(&sha256(&versioned_payload))[..4].to_vec();

//     // Step 5: 生成最终地址 (Base58Check编码)
//     let address_bytes = [versioned_payload, checksum].concat();
//     bs58::encode(address_bytes).into_string()
// }

// // 6. P2WSH 地址
// fn generate_p2wsh_address(witness_script: &[u8]) -> String {
//     // Step 1: 计算脚本哈希 (SHA-256(witness_script))
//     let script_hash = sha256(witness_script);

//     // Step 2: 创建见证程序 (0x00 + 脚本哈希)
//     let witness_program = [b"\x00", &script_hash].concat();

//     // Step 3: 使用 Bech32 编码生成最终地址
//     bech32::encode("bc", witness_program.to_base32()).unwrap()
// }

// fn generate_xonly_pubkey(pubkey: &PublicKey) -> [u8; 32] {
//     let serialized = pubkey.serialize();
//     let mut xonly_pubkey = [0u8; 32];
//     xonly_pubkey.copy_from_slice(&serialized[1..33]); // 取前33个字节中的后32个字节（x 坐标）
//     xonly_pubkey
// }

// fn tweak_xonly_pubkey(xonly_pubkey: &[u8; 32], tweak: &[u8]) -> [u8; 32] {
//     let secp = Secp256k1::new();
//     let pubkey = PublicKey::from_slice(
//         &[0x02]
//             .iter()
//             .chain(xonly_pubkey.iter())
//             .cloned()
//             .collect::<Vec<_>>(),
//     )
//     .expect("Invalid public key");
//     let tweaked = pubkey
//         .add_exp_assign(&secp, tweak)
//         .expect("Tweaking failed");
//     generate_xonly_pubkey(&tweaked)
// }

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use litecoin::{key::Secp256k1, script::Builder, Address, PubkeyHash, ScriptBuf};
    use serde::Serialize;
    use solana_sdk::pubkey;
    use wallet_utils::init_test_log;

    // use crate::instance::dog::generate_p2sh_address;

    use super::derivate_key;

    // legacy
    #[test]
    fn test_generate_p2pkh_address() -> Result<(), Box<dyn std::error::Error>> {
        init_test_log();
        let secp = Secp256k1::new();
        let path = "m/44'/0'/0'/0/0";
        let key = derivate_key(&secp, path)?;
        let pkey = key.private_key.secret_bytes();

        let keypair = key.to_keypair(&secp);
        let pubkey = keypair.public_key();
        let pubkey_str = pubkey.to_string();
        tracing::info!("pubkey: {pubkey_str}");

        let pubkey = pubkey.serialize();
        let network = wallet_types::chain::network::NetworkKind::Mainnet;
        let res = crate::instance::dog::address::generate_p2pkh_address(&pubkey, network);

        println!("res: {res:?}");
        Ok(())
    }

    #[test]
    fn test_generate_p2sh_address() {
        // let secp = Secp256k1::new();
        // let key = derivate_key(&secp);

        // let keypair = key.to_keypair(&secp);
        // let pubkey = keypair.public_key().serialize();
        // let script =
        // let res = generate_p2sh_address(&pubkey, true);

        // println!("res: {res}");
    }

    // 隔离见证（原生）
    #[test]
    fn test_generate_p2wpkh_address() -> Result<(), Box<dyn std::error::Error>> {
        let secp = Secp256k1::new();
        let path = "m/84'/0'/0'/0/0";
        let key = derivate_key(&secp, path)?;

        let keypair = key.to_keypair(&secp);
        let pubkey = keypair.public_key().serialize();
        let network = wallet_types::chain::network::NetworkKind::Mainnet;
        let res = crate::instance::dog::address::generate_p2wpkh_address(&pubkey, network);

        println!("res: {res:?}");
        Ok(())
    }

    // Taproot
    #[test]
    fn test_generate_p2tr_address() -> Result<(), Box<dyn std::error::Error>> {
        let secp = Secp256k1::new();
        let path = "m/86'/0'/0'/0/0";
        let key = derivate_key(&secp, path)?;

        let keypair = key.to_keypair(&secp);
        // let pubkey = keypair.public_key().serialize();
        let network = wallet_types::chain::network::NetworkKind::Mainnet;
        let res = crate::instance::dog::address::generate_p2tr_address(&keypair, &secp, network);

        println!("res: {res:?}");
        Ok(())
    }

    // 隔离见证（兼容）
    #[test]
    fn test_generate_p2sh_p2wpkh_address() -> Result<(), Box<dyn std::error::Error>> {
        let secp = Secp256k1::new();
        let path = "m/49'/0'/0'/0/0";
        let key = derivate_key(&secp, path)?;

        let keypair = key.to_keypair(&secp);
        let pubkey = keypair.public_key().serialize();

        let network = wallet_types::chain::network::NetworkKind::Mainnet;
        let res = crate::instance::dog::address::generate_p2sh_p2wpkh_address(&pubkey, network);

        println!("res: {res}");
        Ok(())
    }

    #[test]
    fn test_generate_p2wsh() {
        let script1 = ScriptBuf::from_hex("52210375e00eb72e29da82b89367947f29ef34afb75e8654f6ea368e0acdfd92976b7c2103a1b26313f430c4b15bb1fdce663207659d8cac749a0e53d70eff01874496feff2103c96d495bfdd5ba4145e3e046fee45e84a8a48ad05bd8dbb395c011a32cf9f88053ae").unwrap();

        let pubkey1 = litecoin::PublicKey::from_str(
            "022b1c8becf58ce0a7db2eaf5666f295c7c8343077e09a0b2666eb51f1cbc08446",
        )
        .unwrap()
        .to_bytes();
        let mut pk_bytes1 = [0_u8; 33];
        pk_bytes1.copy_from_slice(&pubkey1);

        let pubkey2 = litecoin::PublicKey::from_str(
            "02923ae9757390d24e39439d7bd337f1cbfdce38048ee004afd88e1cea099719bf",
        )
        .unwrap()
        .to_bytes();
        let mut pk_bytes2 = [0_u8; 33];
        pk_bytes2.copy_from_slice(&pubkey2);

        let pubkey3 = litecoin::PublicKey::from_str(
            "024a9c26d9c395129c8c097a7b255568410ea9d4c093b229b8c96a25f3435bdc14",
        )
        .unwrap()
        .to_bytes();
        let mut pk_bytes3 = [0_u8; 33];
        pk_bytes3.copy_from_slice(&pubkey3);

        println!("len {}", pubkey1.len());
        let script2 = Builder::new()
            .push_int(2)
            .push_slice(pk_bytes1)
            .push_slice(pk_bytes2)
            .push_slice(pk_bytes3)
            .push_int(3)
            .into_script();

        println!("script1: {}", script2.wscript_hash());
        let address = Address::p2wsh(&script2, litecoin::network::Network::Regtest);

        println!("script1: {script1}");
        println!("script2: {script2}");
        println!("address: {address}");
    }
}

/////

// fn generate_keys() -> (SecretKey, PublicKey) {
//     let secp = Secp256k1::new();
//     let mut rng = OsRng::new().unwrap();
//     let (secret_key, public_key) = secp.generate_keypair(&mut rng);
//     (secret_key, public_key)
// }

// fn is_even_y(pubkey: &PublicKey) -> bool {
//     pubkey.serialize()[64] % 2 == 0
// }

// fn negate_pubkey(pubkey: &PublicKey) -> PublicKey {
//     let mut serialized = pubkey.serialize();
//     serialized[64] = 0xff - serialized[64];
//     PublicKey::from_slice(&serialized).unwrap()
// }

// fn compute_taproot_pubkey(internal_pubkey: &PublicKey) -> PublicKey {
//     *internal_pubkey
// }

// fn convert_bits(data: &[u8], from: u32, to: u32, pad: bool) -> Option<Vec<u8>> {
//     let mut acc = 0;
//     let mut bits = 0;
//     let mut ret = Vec::new();
//     let maxv: u32 = (1 << to) - 1;
//     let max_acc: u32 = (1 << (from + to - 1)) - 1;

//     for value in data {
//         acc = ((acc << from) | (*value as u32)) & max_acc;
//         bits += from;
//         while bits >= to {
//             bits -= to;
//             ret.push(((acc >> bits) & maxv) as u8);
//         }
//     }

//     if pad {
//         if bits > 0 {
//             ret.push(((acc << (to - bits)) & maxv) as u8);
//         }
//     } else if bits >= from || ((acc << (to - bits)) & maxv) != 0 {
//         return None;
//     }

//     Some(ret)
// }

// fn generate_bech32m_address(taproot_pubkey: &PublicKey) -> String {
//     let pubkey_bytes = taproot_pubkey.serialize();
//     let data = convert_bits(&pubkey_bytes[1..], 8, 5, true).unwrap(); // Remove the leading 0x02/0x03 byte
//     let mut bech32_data = vec![u5::try_from_u8(1).unwrap()]; // witness version 1 for Taproot
//     bech32_data.extend(data.into_iter().map(|v| u5::try_from_u8(v).unwrap()));

//     encode("bc", bech32_data, Variant::Bech32m).unwrap()
// }
