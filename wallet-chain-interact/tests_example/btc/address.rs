use bitcoin::{
    bip32::{DerivationPath, Xpriv, Xpub},
    hashes::{ripemd160, sha256, Hash},
    key::{Keypair, Secp256k1},
    secp256k1::All,
    Address, CompressedPublicKey, KnownHrp, Network,
};
use coins_bip39::{English, Mnemonic};
use std::str::FromStr;
use wallet_chain_interact::script::BtcScript;

const NET: Network = Network::Bitcoin;

fn private_key() -> Xpriv {
    let mnemonic = "mnemonic";
    let mnemonic = Mnemonic::<English>::new_from_phrase(mnemonic).unwrap();

    // 生成种子
    let seed = mnemonic.to_seed(Some("")).unwrap();
    let xpriv = Xpriv::new_master(Network::Bitcoin, &seed).unwrap();
    xpriv
}

fn derivation_path() -> DerivationPath {
    let derivation_path =
        DerivationPath::from_str("m/86'/0'/0'/0/0").expect("Invalid derivation path");
    derivation_path
}

fn derivate_key(secp: &Secp256k1<All>) -> Xpriv {
    let xpiri = private_key();
    let path = derivation_path();

    xpiri.derive_priv(&secp, &path).unwrap()
}

#[test]
fn fingerprint() {
    let xpriv = private_key();
    let secp = Secp256k1::new();
    println!("fingerprint: {:?}", xpriv.fingerprint(&secp));

    let xpub = Xpub::from_priv(&secp, &xpriv);

    let binding = xpub.public_key.to_string();
    let xpub_bytes = binding.as_bytes();
    let sha256_hash = sha256::Hash::hash(&xpub_bytes);
    let ripemd160_hash = ripemd160::Hash::hash(sha256_hash.to_string().as_bytes());
    let fingerprint = &ripemd160_hash[..4]; // 取前4个字节作为指纹

    let fingerprint_hex = fingerprint
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    println!("Master fingerprint: {}", fingerprint_hex);
}

// legacy
#[test]
fn addr_p2pkh() {
    let secp = Secp256k1::new();
    let xpriv = derivate_key(&secp);

    let sk = bitcoin::PrivateKey::new(xpriv.private_key, NET);
    let pk = sk.public_key(&secp);

    let addr = Address::p2pkh(pk.pubkey_hash(), NET);
    println!("p2pkh address : {}", addr);
}

#[test]
fn addr_p2sh() {
    let xpriv = private_key();
    let secp = Secp256k1::new();

    let prikey = bitcoin::PrivateKey::new(xpriv.private_key, NET);
    let pk = bitcoin::PublicKey::from_private_key(&secp, &prikey);

    let script = BtcScript::time_lock_script(50000, pk);

    let addr = Address::p2sh(&script, NET).unwrap();
    println!("p2sh address: {}", addr);
}

#[test]
fn addr_p2wpkh() {
    let secp = Secp256k1::new();
    let xpriv = derivate_key(&secp);

    let prikey = bitcoin::PrivateKey::new(xpriv.private_key, NET);
    let compresskey = CompressedPublicKey::from_private_key(&secp, &prikey).unwrap();

    let addr = Address::p2wpkh(&compresskey, NET);
    println!("p2wpkh address: {}", addr);
}

#[test]
fn addr_p2wpsh() {
    let secp = Secp256k1::new();
    let xpriv = derivate_key(&secp);

    let prikey = bitcoin::PrivateKey::new(xpriv.private_key, NET);
    let pk = bitcoin::PublicKey::from_private_key(&secp, &prikey);

    let script = BtcScript::time_lock_script(50000, pk);
    let addr = Address::p2wsh(&script, NET);
    println!("p2wpsh address: {}", addr);
}

#[test]
// 包装地址类型
fn addr_p2shwpkh() {
    let secp = Secp256k1::new();
    let xpriv = derivate_key(&secp);

    let prikey = bitcoin::PrivateKey::new(xpriv.private_key, NET);
    let compresskey: CompressedPublicKey =
        CompressedPublicKey::from_private_key(&secp, &prikey).unwrap();

    let addr = Address::p2shwpkh(&compresskey, NET);
    println!("p2wpsh address: {}", addr);
}

#[test]
fn addr_p2shwsh() {
    let xpriv = private_key();
    let secp = Secp256k1::new();

    let prikey = bitcoin::PrivateKey::new(xpriv.private_key, NET);
    let pk = bitcoin::PublicKey::from_private_key(&secp, &prikey);

    let script = BtcScript::time_lock_script(50000, pk);

    let addr = Address::p2shwsh(&script, NET);
    println!("pw2shwsh address: {}", addr);
}

#[test]
fn addr_p2tr() {
    let secp = Secp256k1::new();
    let xpriv = derivate_key(&secp);

    let prikey = bitcoin::PrivateKey::new(xpriv.private_key, NET);

    let keypair = Keypair::from_secret_key(&secp, &prikey.inner);
    let (internal_key, _parity) = keypair.x_only_public_key();
    println!("internal_key: {internal_key:?}");
    let address = Address::p2tr(&secp, internal_key, None, KnownHrp::Mainnet);
    println!("address.witness_program(): {:?}", address.witness_program());
    println!("p2tr address : {}", address);
}
