use super::{cipher::SymmetricCipher, context::KdfContext, mac::MacCalculator};



pub struct EncryptionPipeline {
    kdf: KdfContext,
    cipher: Box<dyn SymmetricCipher>,
    mac: Box<dyn MacCalculator>,
}

impl EncryptionPipeline {
    pub fn new(kdf: KdfParams) -> Self {
        Self {
            kdf: KdfContext::new(kdf),
            cipher: Box::new(Aes128Ctr),
            mac: Box::new(Keccak256Mac),
        }
    }

    pub fn encrypt(
        &self,
        rng: &mut impl SecureRng,
        data: &[u8],
        password: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), KeystoreError> {
        let salt = rng.generate_bytes(DEFAULT_KEY_SIZE);
        let iv = rng.generate_bytes(DEFAULT_IV_SIZE);
        
        let key = self.kdf.derive_key(password)?;
        let mut ciphertext = data.to_vec();
        self.cipher.encrypt(&key[..16], &iv, &mut ciphertext)?;
        let mac = self.mac.compute(&key, &ciphertext);
        
        Ok((salt, iv, ciphertext, mac))
    }
}