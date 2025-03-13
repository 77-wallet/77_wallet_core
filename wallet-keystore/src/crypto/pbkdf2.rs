use crate::error::crypto::KeystoreError;

use super::{KdfParams, KeyDerivationFunction};

use crate::KdfAlgorithm;
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Pbkdf2Params {
    pub c: u32,
    pub dklen: u8,
    pub prf: String,
    pub salt: crate::utils::HexBytes,
}
