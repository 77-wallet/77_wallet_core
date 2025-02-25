use crate::error::crypto::KeystoreError;
use hmac::Hmac;
use scrypt::{scrypt, Params as ScryptParams};
use sha2::Sha256;

pub trait KeyDerivation {
    fn derive_key(&self, password: &[u8], salt: &[u8]) -> Result<Vec<u8>, KeystoreError>;
}
pub struct ScryptKdf {
    pub params: ScryptParams,
}

impl KeyDerivation for ScryptKdf {
    fn derive_key(&self, password: &[u8], salt: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        let mut key = vec![0u8; self.params.log_n() as usize];
        scrypt(password, salt, &self.params, &mut key)?;
        Ok(key)
    }
}

pub struct Pbkdf2Kdf {
    pub iterations: u32,
    pub dklen: u32,
}

impl KeyDerivation for Pbkdf2Kdf {
    fn derive_key(&self, password: &[u8], salt: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        let mut key = vec![0u8; self.dklen as usize];
        pbkdf2::pbkdf2::<Hmac<Sha256>>(password, salt, self.iterations, &mut key);
        Ok(key)
    }
}
