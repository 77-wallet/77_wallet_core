use super::json::KeystoreJson;

// 文件管理层（处理路径和序列化）
pub struct KeystoreFile {
    path: std::path::PathBuf,
    engine: super::engine::KeystoreEngine,
}

impl KeystoreFile {
    pub(crate) fn new(path: std::path::PathBuf, engine: super::engine::KeystoreEngine) -> Self {
        Self { path, engine }
    }

    pub fn save<T: AsRef<[u8]>, R: rand::Rng + rand::CryptoRng>(
        &self,
        rng: &mut R,
        data: &T,
        password: &[u8],
    ) -> Result<KeystoreJson, crate::Error> {
        // 序列化业务数据
        // let serialized = wallet_utils::serde_func::serde_to_vec(data)?;

        // 加密核心数据
        let crypto = self.engine.encrypt(rng, &data, password)?;

        // 构建完整文件结构
        // let keystore = KeystoreJson {
        //     version: 2,
        //     uuid: Uuid::new_v4(),
        //     crypto,
        //     metadata: json!({"type": std::any::type_name::<T>()}),
        // };

        let contents = wallet_utils::serde_func::serde_to_string(&crypto)?;
        // 写入文件
        wallet_utils::file_func::write_all(&self.path, contents.as_bytes())?;

        Ok(crypto)
    }
}
