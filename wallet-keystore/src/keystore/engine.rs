use rand::{CryptoRng, Rng};
use uuid::Uuid;

use crate::crypto::kdfs::KeyDerivation;

use super::{
    cipher::SymmetricCipher,
    json::{CipherparamsJson, CryptoJson, KeystoreJson},
    mac::MacCalculator,
};

const DEFAULT_CIPHER: &str = "aes-128-ctr";
const DEFAULT_KEY_SIZE: usize = 32usize;
const DEFAULT_IV_SIZE: usize = 16usize;

/// 核心加密层（不涉及文件操作）
pub struct KeystoreEngine {
    kdf: Box<dyn KeyDerivation>,
    // cipher: Box<dyn SymmetricCipher>,
    // mac: Box<dyn MacCalculator>,
}

impl KeystoreEngine {
    pub fn new(
        kdf: Box<dyn KeyDerivation>,
        // cipher: Box<dyn SymmetricCipher>,
        // mac: Box<dyn MacCalculator>,
    ) -> Self {
        Self {
            kdf,
            //  cipher,
            // mac,
        }
    }

    pub fn encrypt<T: AsRef<[u8]>, R: Rng + CryptoRng>(
        &self,
        rng: &mut R,
        data: &T,
        password: &[u8],
    ) -> Result<KeystoreJson, crate::Error> {
        // let salt = generate_random_bytes(rng, DEFAULT_KEY_SIZE);
        let iv = crate::generate_random_bytes(rng, DEFAULT_IV_SIZE);

        let key = self.kdf.derive_key(password)?;
        // let mut ciphertext = data.as_ref().to_vec();
        // let ciphertext = wallet_utils::serde_func::serde_to_vec(data)?;

        let data = super::cipher::Aes128Ctr::encrypt(&key[..16], &iv[..16], data.as_ref())?;
        // self.cipher.encrypt(&key, &iv, &ciphertext)?;

        // Calculate the MAC.
        let mac = super::mac::Keccak256Mac.compute(&key, data.as_ref());
        // let mac = self.mac.compute(&key, &ciphertext);

        let id = Uuid::new_v4();
        // let name = if let Some(name) = name {
        //     name.to_string()
        // } else {
        //     id.to_string()
        // };

        Ok(KeystoreJson {
            crypto: CryptoJson {
                cipher: String::from(DEFAULT_CIPHER),
                cipherparams: CipherparamsJson { iv },
                ciphertext: data,
                kdf: self.kdf.algorithm(),
                // kdfparams: KdfParams::Scrypt(ScryptParams {
                //     dklen: DEFAULT_KDF_PARAMS_DKLEN,
                //     n: 2u32.pow(DEFAULT_KDF_PARAMS_LOG_N as u32),
                //     p: DEFAULT_KDF_PARAMS_P,
                //     r: DEFAULT_KDF_PARAMS_R,
                //     salt,
                // }),
                kdfparams: self.kdf.params(),
                mac: mac.to_vec(),
            },
            id,
            version: 3,
        })
    }

    pub fn decrypt(
        &self,
        password: &[u8],
        keystore: KeystoreJson,
    ) -> Result<Vec<u8>, crate::Error> {
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
        Ok(data)
    }
}
