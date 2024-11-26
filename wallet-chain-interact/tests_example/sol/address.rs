use coins_bip39::{English, Mnemonic};
use solana_sdk::{
    derivation_path::DerivationPath, signature::keypair_from_seed_and_derivation_path,
    signer::Signer,
};
fn seed() -> [u8; 64] {
    let mnemonic = "victory member rely dirt treat woman boring tomato two hollow erosion drop";
    let mnemonic = Mnemonic::<English>::new_from_phrase(mnemonic).unwrap();

    // 生成种子
    let seed = mnemonic.to_seed(Some("")).unwrap();
    seed
}

#[test]
fn test_addr1() {
    let path = "m/44'/501'/0'/0'";
    let path = DerivationPath::from_absolute_path_str(&path).unwrap();
    let keypair = keypair_from_seed_and_derivation_path(&seed(), Some(path)).unwrap();

    let pubkey = keypair.pubkey();
    // println!("private_key: {}", hex::encode(keypair.secret()));
    println!("address: {}", pubkey);

    assert_eq!(
        "GE93MHXVvnsbhxu7Ttpp7zTiipJeCX3QFXueSK2dCJe6",
        pubkey.to_string()
    )
}

#[test]
fn test_addr2() {
    let path = "m/44'/501'/1'/0'";
    let path = DerivationPath::from_absolute_path_str(&path).unwrap();
    let keypair = keypair_from_seed_and_derivation_path(&seed(), Some(path)).unwrap();

    let pubkey = keypair.pubkey();
    println!("address: {}", pubkey);

    assert_eq!(
        "MmqgDWhS59oXWVuVtogpvj6k5RLny2ZHCGwDQX1yqkC",
        pubkey.to_string()
    )
}

#[test]
fn test_addr3() {
    let path = "m/44'/501'/2'/0'";
    let path = DerivationPath::from_absolute_path_str(&path).unwrap();
    let keypair = keypair_from_seed_and_derivation_path(&seed(), Some(path)).unwrap();

    let pubkey = keypair.pubkey();
    println!("address: {}", pubkey);

    assert_eq!(
        "Ey3PmUxYJXK6DrtNSq47aE86tcMf9u6EbM89Dh76etPt",
        pubkey.to_string()
    )
}

#[test]
fn test_key() {
    let key1 = "13924d06e80f229a75a4f7b4e434b47b6532c4f15e2ffd68ebc2b79db05bb237";

    // let str = "4JsE64VDotmqJMJRQz53XDAxVCwhmm7bdhybNVMA4fu12WB2etB3QgPwfA3FUFGf2M8RzN9NLF9cJX1sV5ygpQvP";
    // let test = solana_sdk::signature::Keypair::from_base58_string(&str);

    // // A9gBqKMQDWUYNiHHpHakSEsztKuxxN838EWGuG2WKc6F

    // // 0xd8be6e06ce65c85b3c9f4d04dcbe68ffd72524e60d90b069b535251d9febe4be
    // println!("test: {}", test.pubkey().to_string());

    // let key2 = solana_sdk::signature::Keypair  ::from_bytes(&key_bytes).unwrap();

    let key1 = solana_sdk::bs58::encode(key1).into_string();

    assert_eq!(
        key1,
        "PhKgs4sb76HtfzZv2N5ZxFfupPtKQghRdE3c8q2UW65JokknVwtPvsGnzQYtURAtf6Z5u1DFVtNxqzwkMJJ7VwQ",
    );
}
