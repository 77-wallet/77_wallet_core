use aes::cipher::BlockEncryptMut as _;
use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
use base64::prelude::*;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

#[derive(Debug, Clone)]
pub struct AesCbcCryptor {
    pub key: Vec<u8>,
    pub iv: [u8; 16],
}

impl AesCbcCryptor {
    pub fn new(key: &str, iv: &str) -> Self {
        Self {
            key: key.as_bytes().to_vec(),
            iv: iv.as_bytes().try_into().expect("IV必须是16字节"),
        }
    }

    fn create_cipher<T: KeyIvInit>(&self) -> Result<T, crate::Error> {
        T::new_from_slices(&self.key, &self.iv).map_err(|e| crate::Error::Crypto(e.into()))
    }

    pub fn decrypt(&self, encrypted_data: &str) -> Result<serde_json::Value, crate::Error> {
        let encrypted_data = BASE64_STANDARD
            .decode(encrypted_data)
            .map_err(|e| crate::Error::Crypto(e.into()))?;

        let decryptor: Aes128CbcDec = self.create_cipher()?;
        let buf_size = encrypted_data.len();
        let mut buf = vec![0u8; buf_size];
        let decrypted_data = decryptor
            .decrypt_padded_b2b_mut::<Pkcs7>(&encrypted_data, &mut buf)
            .map_err(|e| crate::Error::Crypto(e.into()))?;

        let decrypted_str = String::from_utf8_lossy(decrypted_data);
        crate::serde_func::serde_from_str(&decrypted_str)
    }

    pub fn encrypt(&self, raw_data: &str) -> Result<String, crate::Error> {
        let data = raw_data.as_bytes();
        let encryptor: Aes128CbcEnc = self.create_cipher()?;

        let buf_size = data.len() + 16;
        let mut buf = vec![0u8; buf_size];
        let encrypted_data = encryptor
            .encrypt_padded_b2b_mut::<Pkcs7>(data, &mut buf)
            .map_err(|e| crate::Error::Crypto(e.into()))?;

        Ok(BASE64_STANDARD.encode(encrypted_data))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_aes() {
        let iv = "0000000000000000";
        let key = "u3es1w0suq515aiw";

        let encrypted = "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";

        let decrypter = AesCbcCryptor::new(key, iv);
        let _res = decrypter.decrypt(encrypted).unwrap();
        // println!("Decrypted: {}", res);

        // let res = decrypter.encrypt(&res).unwrap();
        println!("encrypted: {:#?}", _res);

        // assert_eq!(encrypted, res);
    }
}
