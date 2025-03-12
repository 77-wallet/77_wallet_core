use aes::{
    cipher::{InnerIvInit, KeyInit, StreamCipherCore},
    Aes128,
};

use crate::error::crypto::KeystoreError;

pub trait SymmetricCipher {
    fn encrypt(key: &[u8], iv: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, KeystoreError>;
    fn decrypt(key: &[u8], iv: &[u8], ciphertext: &mut [u8]) -> Result<(), KeystoreError>;
}

pub struct Aes128Ctr {
    inner: ctr::CtrCore<Aes128, ctr::flavors::Ctr128BE>,
}

impl Aes128Ctr {
    pub(crate) fn new(key: &[u8], iv: &[u8]) -> Result<Self, KeystoreError> {
        let cipher = aes::Aes128::new_from_slice(key)?;
        let inner = ctr::CtrCore::inner_iv_slice_init(cipher, iv)?;
        Ok(Self { inner })
    }

    pub(crate) fn apply_keystream(self, buf: &mut [u8]) {
        self.inner.apply_keystream_partial(buf.into());
    }
}

impl SymmetricCipher for Aes128Ctr {
    fn encrypt(key: &[u8], iv: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        let mut buf = plaintext.to_vec();
        let cipher = Aes128Ctr::new(key, iv)?;
        cipher.apply_keystream(&mut buf);
        Ok(buf)
    }

    fn decrypt(key: &[u8], iv: &[u8], ciphertext: &mut [u8]) -> Result<(), KeystoreError> {
        let cipher = Aes128Ctr::new(key, iv)?;
        cipher.apply_keystream(ciphertext);
        Ok(())
    }
}
