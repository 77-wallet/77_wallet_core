use rand::{CryptoRng, Rng};

use crate::crypto::engine::CryptoEngine;

pub struct CryptoService<E: CryptoEngine> {
    engine: E,
}

impl<E: CryptoEngine> CryptoService<E> {
    pub fn new(engine: E) -> Self {
        Self { engine }
    }
    pub fn encrypt<R: Rng + CryptoRng>(
        &self,
        rng: &mut R,
        data: &[u8],
        password: &[u8],
    ) -> Result<E::Data, crate::Error>
    where
        E::Data: serde::Serialize,
    {
        let encrypted = self.engine.encrypt(rng, &data, password)?;
        Ok(encrypted)
    }

    pub fn decrypt_from_string(&self, password: &[u8], input: &str) -> Result<Vec<u8>, crate::Error>
    where
        E::Data: for<'de> serde::Deserialize<'de>,
    {
        let encrypted: E::Data = wallet_utils::serde_func::serde_from_str(input)?;
        self.engine.decrypt(password, encrypted)
    }
}
