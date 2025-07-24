use crate::{
    KdfAlgorithm, KeystoreJson,
    keystore::{engine::KeystoreEngine, factory::KdfFactory},
};
const DEFAULT_KEY_SIZE: usize = 32usize;

pub struct EncryptedData {
    pub encrypted: String,
    // pub salt: Vec<u8>,
    // pub algorithm: KdfAlgorithm,
}

impl EncryptedData {
    pub fn encrypt<R: rand::Rng + rand::CryptoRng>(
        rng: &mut R,
        plaintext: &[u8],
        password: &[u8],
        algorithm: &KdfAlgorithm,
    ) -> Result<Self, crate::Error> {
        let salt = crate::generate_random_bytes(rng, DEFAULT_KEY_SIZE);
        let kdf = KdfFactory::create(algorithm, &salt)?;
        let engine = KeystoreEngine::new(kdf);
        let encrypted = engine.encrypt(rng, &plaintext, password)?;
        let contents = wallet_utils::serde_func::serde_to_string(&encrypted)?;
        Ok(Self {
            encrypted: contents,
            // salt,
            // algorithm: algorithm.clone(),
        })
    }

    pub fn decrypt(encrypted: &str, password: &[u8]) -> Result<Vec<u8>, crate::Error> {
        let keystore: KeystoreJson = wallet_utils::serde_func::serde_from_str(encrypted)?;
        let kdf = KdfFactory::create_from_file(&keystore)?;
        let engine = KeystoreEngine::new(kdf);
        engine.decrypt(password, keystore)
    }
}
