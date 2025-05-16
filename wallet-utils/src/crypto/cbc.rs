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

        let encrypted = "4ygdaLBQvQpjRc9EAZlLB87Go2heIufZS4sJuGIgdpdvq6k7HwS3YCRQXnEUbPVT+TBLxby8y+LlJ/cbZ0N1cw==";

        let decrypter = AesCbcCryptor::new(key, iv);
        let _res = decrypter.decrypt(encrypted).unwrap();
        // println!("Decrypted: {}", res);

        // let res = decrypter.encrypt(&res).unwrap();
        println!("encrypted: {:#?}", _res);

        // assert_eq!(encrypted, res);
    }
}
