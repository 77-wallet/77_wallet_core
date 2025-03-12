use crate::{error::crypto::KeystoreError, keystore::factory::Pbkdf2Params};

use super::KeyDerivationFunction;

use crate::{keystore::factory::KdfParams, KdfAlgorithm};
use hmac::Hmac;
use sha2::Sha256;

pub struct Pbkdf2Kdf {
    pub params: Pbkdf2Params,
}

impl Pbkdf2Kdf {
    pub fn new(params: Pbkdf2Params) -> Self {
        Self { params }
    }
}

impl KeyDerivationFunction for Pbkdf2Kdf {
    fn derive_key(&self, password: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        let mut key = vec![0u8; self.params.dklen as usize];
        pbkdf2::pbkdf2::<Hmac<Sha256>>(password, &self.params.salt, self.params.c, &mut key);
        Ok(key)
    }

    fn params(&self) -> KdfParams {
        KdfParams::Pbkdf2(self.params.clone())
    }

    fn algorithm(&self) -> KdfAlgorithm {
        KdfAlgorithm::Pbkdf2
    }
}
