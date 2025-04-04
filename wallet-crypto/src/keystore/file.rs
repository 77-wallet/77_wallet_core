use super::json::KeystoreJson;

// 文件管理层（处理路径和序列化）
pub struct KeystoreFile<P: AsRef<std::path::Path>> {
    path: P,
    engine: super::engine::KeystoreEngine,
}

impl<P: AsRef<std::path::Path>> KeystoreFile<P> {
    pub(crate) fn new(path: P, engine: super::engine::KeystoreEngine) -> Self {
        Self { path, engine }
    }

    pub fn save<T: AsRef<[u8]>, R: rand::Rng + rand::CryptoRng>(
        &self,
        rng: &mut R,
        data: &T,
        password: &[u8],
    ) -> Result<KeystoreJson, crate::Error> {
        // 加密核心数据
        let crypto = self.engine.encrypt(rng, data, password)?;

        let contents = wallet_utils::serde_func::serde_to_string(&crypto)?;
        // 写入文件
        wallet_utils::file_func::create_file(&self.path)?;
        wallet_utils::file_func::write_all(&self.path, contents.as_bytes())?;

        Ok(crypto)
    }
}
