use crate::{
    error::crypto::KeystoreError,
    keystore::factory::{KdfParams, Pbkdf2Params, ScryptParams},
};
use hmac::Hmac;
use scrypt::{scrypt, Params as ScryptParams_};
use sha2::Sha256;

pub trait KeyDerivation {
    fn derive_key(&self, password: &[u8], salt: &[u8]) -> Result<Vec<u8>, KeystoreError>;

    fn params(&self) -> KdfParams;
}
pub struct ScryptKdf {
    pub params: ScryptParams,
}

impl ScryptKdf {
    pub fn new(params: ScryptParams) -> Self {
        Self { params }
    }
}

impl KeyDerivation for ScryptKdf {
    fn derive_key(&self, password: &[u8], salt: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        let mut key = vec![0u8; self.params.dklen as usize];
        let log_n = super::kdf::log2(self.params.n) as u8;
        let scrypt_params = ScryptParams_::new(log_n, self.params.r, self.params.p)?;
        scrypt(password, salt, &scrypt_params, &mut key)?;
        Ok(key)
    }

    fn params(&self) -> KdfParams {
        KdfParams::Scrypt(self.params.clone())
    }
}

pub struct Pbkdf2Kdf {
    pub params: Pbkdf2Params,
}

impl Pbkdf2Kdf {
    pub fn new(params: Pbkdf2Params) -> Self {
        Self { params }
    }
}

impl KeyDerivation for Pbkdf2Kdf {
    fn derive_key(&self, password: &[u8], salt: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        let mut key = vec![0u8; self.params.dklen as usize];
        pbkdf2::pbkdf2::<Hmac<Sha256>>(password, salt, self.params.c, &mut key);
        Ok(key)
    }

    fn params(&self) -> KdfParams {
        KdfParams::Pbkdf2(self.params.clone())
    }
}

#[allow(unused_assignments)]
pub(crate) fn log2(mut n: u32) -> u32 {
    let mut result = 0;
    if (n & 0xffff0000) != 0 {
        result += 16;
        n >>= 16;
    }
    if (n & 0x0000ff00) != 0 {
        result += 8;
        n >>= 8;
    }
    if (n & 0x000000f0) != 0 {
        result += 4;
        n >>= 4;
    }
    if (n & 0x0000000c) != 0 {
        result += 2;
        n >>= 2;
    }
    if (n & 0x00000002) != 0 {
        result += 1;
        n >>= 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::crypto::kdf::log2;

    #[test]
    fn test_log2() {
        let n = 8192;
        println!("log_n: {}", log2(n));
    }
}
