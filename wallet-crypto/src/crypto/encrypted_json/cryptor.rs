use crate::EncryptedJson;

/// 加密能力
pub trait EncryptedJsonGenerator {
    const DEFAULT_KEY_SIZE: usize;
    fn generate(&mut self, password: &[u8], data: &[u8]) -> Result<EncryptedJson, crate::Error>;
}

/// 解密能力
pub trait EncryptedJsonDecryptor {
    fn decrypt(&self, password: &[u8], encrypted: &str) -> Result<Vec<u8>, crate::Error>;
}
