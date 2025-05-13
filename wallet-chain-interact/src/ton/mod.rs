use crate::types::ChainPrivateKey;
use tonlib_core::wallet::mnemonic::KeyPair;

pub mod address;
pub mod chain;
pub mod consts;
pub mod errors;
pub mod operations;
pub mod params;
pub mod protocol;
pub mod provider;

pub use tonlib_core::cell::Cell;

pub fn get_keypair(key: ChainPrivateKey) -> crate::Result<KeyPair> {
    let sk = ed25519_dalek_bip32::SecretKey::from_bytes(&key.to_bytes()?)
        .map_err(|_e| crate::Error::SignError(format!("ton parse keypair from error")))?;

    let pk = ed25519_dalek_bip32::PublicKey::from(&sk);

    let mut sk = sk.as_bytes().to_vec();
    let pk = pk.as_bytes().to_vec();
    sk.extend(&pk);

    Ok(KeyPair {
        secret_key: sk,
        public_key: pk,
    })
}
