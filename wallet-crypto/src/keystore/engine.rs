use rand::{CryptoRng, Rng};
use uuid::Uuid;

use crate::{crypto::engine::CryptoEngine, kdf::KeyDerivationFunction};

use super::{
    cipher::SymmetricCipher,
    json::{CipherparamsJson, CryptoJson, KeystoreJson},
    mac::MacCalculator,
};

const DEFAULT_CIPHER: &str = "aes-128-ctr";
const DEFAULT_IV_SIZE: usize = 16usize;

/// 核心加密层（不涉及文件操作）
pub struct KdfCryptoEngine {
    kdf: Box<dyn KeyDerivationFunction>,
}

impl CryptoEngine for KdfCryptoEngine {
    type Data = KeystoreJson;

    fn encrypt<T: AsRef<[u8]>, R: Rng + CryptoRng>(
        &self,
        rng: &mut R,
        data: &T,
        password: &[u8],
    ) -> Result<Self::Data, crate::Error> {
        let iv = crate::generate_random_bytes(rng, DEFAULT_IV_SIZE);

        let key = self.kdf.derive_key(password)?;

        let data = super::cipher::Aes128Ctr::encrypt(&key[..16], &iv[..16], data.as_ref())?;

        // Calculate the MAC.
        let mac = super::mac::Keccak256Mac.compute(&key, data.as_ref());

        let id = Uuid::new_v4();

        Ok(KeystoreJson {
            crypto: CryptoJson {
                cipher: String::from(DEFAULT_CIPHER),
                cipherparams: CipherparamsJson { iv: iv.into() },
                ciphertext: data.into(),
                kdf: self.kdf.algorithm(),
                kdfparams: self.kdf.params(),
                mac: mac.into(),
            },
            id,
            version: 3,
        })
    }

    fn decrypt(&self, password: &[u8], keystore: Self::Data) -> Result<Vec<u8>, crate::Error> {
        let key = self.kdf.derive_key(password)?;
        let derived_mac = super::mac::Keccak256Mac.compute(&key, &keystore.crypto.ciphertext);

        if derived_mac.as_slice() != keystore.crypto.mac.as_slice() {
            return Err(crate::error::crypto::KeystoreError::MacMismatch.into());
        }
        // Decrypt the private key bytes using AES-128-CTR
        let mut data = keystore.crypto.ciphertext;
        super::cipher::Aes128Ctr::decrypt(
            &key[..16],
            &keystore.crypto.cipherparams.iv[..16],
            &mut data,
        )?;
        Ok(data.0)
    }
}

impl KdfCryptoEngine {
    pub fn new(kdf: Box<dyn KeyDerivationFunction>) -> Self {
        Self { kdf }
    }
}
