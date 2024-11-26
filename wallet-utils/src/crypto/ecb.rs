use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut as _, BlockEncryptMut as _, KeyInit as _};
use hex::{decode as hex_decode, encode as hex_encode};
use std::error::Error;

type Aes128EcbDec = ecb::Decryptor<aes::Aes128>;
type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;

/// AES-128 ECB 加密和解密结构体
pub struct Aes128EcbCryptor {
    key: [u8; 16], // 16 字节密钥
}

impl Aes128EcbCryptor {
    /// 创建新的 AES-128 ECB 结构体
    ///
    /// # 参数
    /// - `key`: 16 字节的密钥字符串
    ///
    /// # 返回
    /// 结构体实例
    pub fn new(key: &str) -> Result<Self, Box<dyn Error>> {
        if key.len() != 16 {
            return Err("Key must be 16 bytes long".into());
        }

        let key_bytes = key.as_bytes();
        let mut key_array = [0u8; 16];
        key_array.copy_from_slice(key_bytes);

        Ok(Self { key: key_array })
    }

    /// 将字节数组转换为十六进制字符串
    fn byte2hex(&self, bytes: &[u8]) -> String {
        hex_encode(bytes).to_ascii_uppercase()
    }

    /// 将十六进制字符串转换为字节数组
    fn hex2byte(&self, hex_str: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let bytes = hex_decode(hex_str)?;
        Ok(bytes)
    }

    /// AES-128 ECB 加密
    ///
    /// # 参数
    /// - `src`: 明文字符串
    ///
    /// # 返回
    /// 加密后的十六进制字符串
    pub fn encrypt(&self, src: &str) -> Result<String, Box<dyn Error>> {
        // 创建 AES-128 ECB 加密器
        let cipher = Aes128EcbEnc::new_from_slice(&self.key)?;

        let data = src.as_bytes();
        let mut buffer = data.to_vec();
        buffer.resize(((buffer.len() / 16) + 1) * 16, 0); // 预留填充空间

        // 执行加密
        let ciphertext = cipher
            .encrypt_padded_mut::<Pkcs7>(&mut buffer, data.len())
            .map_err(|e| format!("Encryption error: {}", e))?;

        Ok(self.byte2hex(ciphertext))
    }

    /// AES-128 ECB 解密
    ///
    /// # 参数
    /// - `src`: 加密后的十六进制字符串
    ///
    /// # 返回
    /// 解密后的明文字符串
    pub fn decrypt(&self, src: &str) -> Result<String, Box<dyn Error>> {
        // 将十六进制字符串转换为字节数组
        let ciphertext = self.hex2byte(src)?;

        // 创建 AES-128 ECB 解密器
        let cipher = Aes128EcbDec::new_from_slice(&self.key)?;

        let mut buffer = ciphertext.clone();

        // 执行解密
        let decrypted_data = cipher
            .decrypt_padded_mut::<Pkcs7>(&mut buffer)
            .map_err(|e| format!("Decryption error: {}", e))?;

        // 将解密后的字节数组转换为字符串并返回
        let decrypted_str = String::from_utf8(decrypted_data.to_vec())?;
        Ok(decrypted_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_client_id() -> Result<(), Box<dyn Error>> {
        let app_id = "bc7f694ee0a9488cada7d9308190fe45";
        let sn = "frank";
        let device_type = "Android";
        let client_id = format!("{}_{}_{}", app_id, sn, device_type);
        let key = "ada7d9308190fe45"; // 16字节密钥

        let aes = Aes128EcbCryptor::new(key)?;

        // 加密
        let encrypted = aes.encrypt(&client_id)?;
        println!("Encrypted (Hex): {}", encrypted);

        // 解密
        let decrypted = aes.decrypt(&encrypted)?;
        println!("Decrypted: {}", decrypted);

        // 验证解密结果与原文相同
        assert_eq!(client_id, decrypted);

        let encrypted_res = "242B034534A5D2CBBC3409DFECA15FBC61746B9A7307E6EDD2AEC6A4D77BDB40AB472963058A3A65B7D9578CF3000B86";
        assert_eq!(encrypted_res, encrypted);

        Ok(())
    }

    #[test]
    fn test_encrypt_decrypt_with_padding() -> Result<(), Box<dyn Error>> {
        let key = "abcdef1234567890"; // 16字节密钥
        let plaintext = "This is a longer plaintext message that is not a multiple of 16 bytes.";

        let aes = Aes128EcbCryptor::new(key)?;

        // 加密
        let encrypted = aes.encrypt(plaintext)?;
        println!("Encrypted (Hex): {}", encrypted);

        // 解密
        let decrypted = aes.decrypt(&encrypted)?;
        println!("Decrypted: {}", decrypted);

        // 验证解密结果与原文相同
        assert_eq!(plaintext, decrypted);

        Ok(())
    }

    #[test]
    fn test_encrypt_with_invalid_key_length() {
        let key = "shortkey"; // 无效的密钥长度
        let plaintext = "Hello, AES Encryption!";

        let result = Aes128EcbCryptor::new(key).and_then(|aes| aes.encrypt(plaintext));

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_invalid_key_length() {
        let key = "shortkey"; // 无效的密钥长度
        let encrypted = "3ad77bb40d7a3660a89ecaf32466ef97";

        let result = Aes128EcbCryptor::new(key).and_then(|aes| aes.decrypt(encrypted));

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_invalid_hex() {
        let key = "1234567890abcdef"; // 16字节密钥
        let encrypted = "invalidhexstring";

        let result = Aes128EcbCryptor::new(key).and_then(|aes| aes.decrypt(encrypted));

        assert!(result.is_err());
    }
}
