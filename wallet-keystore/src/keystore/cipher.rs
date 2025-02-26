use aes::{
    cipher::{InnerIvInit, KeyInit, StreamCipherCore},
    Aes128,
};

use crate::error::crypto::KeystoreError;

pub trait SymmetricCipher {
    fn encrypt(key: &[u8], iv: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, KeystoreError>
    where
        Self: Sized;
    fn decrypt(key: &[u8], iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, KeystoreError>
    where
        Self: Sized;
}

pub struct Aes128Ctr {
    inner: ctr::CtrCore<Aes128, ctr::flavors::Ctr128BE>,
}

impl Aes128Ctr {
    fn new(key: &[u8], iv: &[u8]) -> Result<Self, KeystoreError> {
        let cipher = aes::Aes128::new_from_slice(key)?;
        let inner = ctr::CtrCore::inner_iv_slice_init(cipher, iv)?;
        Ok(Self { inner })
    }

    fn apply_keystream(self, buf: &mut [u8]) {
        self.inner.apply_keystream_partial(buf.into());
    }
}

impl SymmetricCipher for Aes128Ctr {
    fn encrypt(key: &[u8], iv: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, KeystoreError>
    where
        Self: Sized,
    {
        let mut buf = plaintext.to_vec();
        let cipher = Aes128Ctr::new(key, iv)?;
        cipher.apply_keystream(&mut buf);
        Ok(buf)
    }

    fn decrypt(key: &[u8], iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, KeystoreError>
    where
        Self: Sized,
    {
        Aes128Ctr::encrypt(key, iv, ciphertext) // CTR 模式加解密相同
    }
}
