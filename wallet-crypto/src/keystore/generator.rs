use crate::crypto::encrypted_json::cryptor::{EncryptedJsonDecryptor, EncryptedJsonGenerator};
use crate::crypto::encrypted_json::encrypted::EncryptedJson;
use crate::crypto::encrypted_json::service::CryptoService;

use crate::kdf::factory::KdfFactory;
use crate::keystore::engine::KdfCryptoEngine;
use crate::{KdfAlgorithm, generate_random_bytes};

pub struct KeystoreJsonGenerator<R: rand::Rng + rand::CryptoRng> {
    rng: R,
    algorithm: KdfAlgorithm,
}

impl<R: rand::Rng + rand::CryptoRng> KeystoreJsonGenerator<R> {
    pub fn new(rng: R, algorithm: KdfAlgorithm) -> Self {
        Self { rng, algorithm }
    }
}

impl<R: rand::Rng + rand::CryptoRng> EncryptedJsonGenerator for KeystoreJsonGenerator<R> {
    const DEFAULT_KEY_SIZE: usize = 32usize;

    fn generate(&mut self, password: &[u8], data: &[u8]) -> Result<EncryptedJson, crate::Error> {
        let salt = generate_random_bytes(&mut self.rng, Self::DEFAULT_KEY_SIZE);
        let kdf = KdfFactory::create(&self.algorithm, &salt)?;
        let engine = KdfCryptoEngine::new(kdf);
        CryptoService::new(engine).encrypt(&mut self.rng, data, password)
    }
}

pub struct KeystoreJsonDecryptor;

impl EncryptedJsonDecryptor for KeystoreJsonDecryptor {
    fn decrypt(&self, password: &[u8], encrypted: &str) -> Result<Vec<u8>, crate::Error> {
        let keystore: EncryptedJson = wallet_utils::serde_func::serde_from_str(encrypted)?;
        let kdf = KdfFactory::create_from_encrypted_data(&keystore)?;
        let engine = KdfCryptoEngine::new(kdf);

        CryptoService::new(engine).decrypt_from_string(password, encrypted)
    }
}
