use rand::{CryptoRng, Rng};

use crate::{crypto::kdf::KeyDerivation, error::crypto::KeystoreError};

use super::{cipher::SymmetricCipher, json::KeystoreJson, mac::MacCalculator};

pub struct KeystoreBuilder {
    kdf: Box<dyn KeyDerivation>,
    cipher: Box<dyn SymmetricCipher>,
    mac: Box<dyn MacCalculator>,
}

impl KeystoreBuilder {
    pub fn new(
        kdf: impl KeyDerivation + 'static,
        cipher: impl SymmetricCipher + 'static,
        mac: impl MacCalculator + 'static,
    ) -> Self {
        Self {
            kdf: Box::new(kdf),
            cipher: Box::new(cipher),
            mac: Box::new(mac),
        }
    }

    pub fn encrypt<R: Rng + CryptoRng>(
        &self,
        rng: &mut R,
        data: &[u8],
        password: &[u8],
    ) -> Result<KeystoreJson, KeystoreError> {
        // 实现加密流程
        todo!()
    }
}
