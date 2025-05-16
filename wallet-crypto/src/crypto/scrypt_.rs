use crate::{KdfAlgorithm, error::crypto::KeystoreError, utils::HexBytes};

use super::{KdfParams, KeyDerivationFunction};

use scrypt::Params as ScryptParams_;

pub struct ScryptKdf {
    pub params: ScryptParams,
}

impl ScryptKdf {
    pub fn new(params: ScryptParams) -> Self {
        Self { params }
    }
}

impl KeyDerivationFunction for ScryptKdf {
    fn derive_key(&self, password: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        let start = std::time::Instant::now();
        let mut key = vec![0u8; self.params.dklen as usize];
        let log_n = super::log2(self.params.n) as u8;
        let scrypt_params = ScryptParams_::new(log_n, self.params.r, self.params.p)?;
        scrypt::scrypt(password, &self.params.salt, &scrypt_params, &mut key)?;
        tracing::info!("scrypt cost: {:?}", start.elapsed());
        Ok(key)
    }

    fn params(&self) -> KdfParams {
        KdfParams::Scrypt(self.params.clone())
    }

    fn algorithm(&self) -> KdfAlgorithm {
        KdfAlgorithm::Scrypt
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScryptParams {
    pub dklen: u8,
    pub n: u32,
    pub p: u32,
    pub r: u32,
    pub salt: HexBytes,
}

impl Default for ScryptParams {
    fn default() -> Self {
        Self {
            dklen: 32u8,
            n: 2u32.pow(10u8 as u32),
            r: 8,
            p: 1,
            salt: HexBytes(vec![]),
        }
    }
}

impl ScryptParams {
    pub fn new(dklen: u8, n: u32, r: u32, p: u32, salt: &[u8]) -> Self {
        Self {
            dklen,
            n,
            r,
            p,
            salt: HexBytes(salt.to_vec()),
        }
    }

    pub(crate) fn with_salt(mut self, salt: &[u8]) -> Self {
        self.salt = HexBytes(salt.to_vec());
        self
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_log2() {
        let n = 8192;
        println!("log_n: {}", crate::crypto::log2(n));
    }
}
