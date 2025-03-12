use crate::{
    error::crypto::KeystoreError,
    keystore::factory::{KdfParams, ScryptParams},
    KdfAlgorithm,
};

use super::KeyDerivation;

use scrypt::Params as ScryptParams_;

pub struct ScryptKdf {
    pub params: ScryptParams,
}

impl ScryptKdf {
    pub fn new(params: ScryptParams) -> Self {
        Self { params }
    }
}

impl KeyDerivation for ScryptKdf {
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
