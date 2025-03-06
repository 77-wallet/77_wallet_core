use rand::{CryptoRng, RngCore};

use crate::KdfAlgorithm;

const DEFAULT_KEY_SIZE: usize = 32usize;

use super::{engine::KeystoreEngine, factory::KdfFactory, file::KeystoreFile, json::KeystoreJson};

pub struct KeystoreBuilder<M, P: AsRef<std::path::Path>> {
    path: P,
    password: Vec<u8>,
    crypto_mode: M,
}

pub struct EncryptMode<R, D> {
    rng: R,
    algorithm: KdfAlgorithm,
    file_name: String,
    data: D,
}

pub struct DecryptMode {}

impl<R, D, P> KeystoreBuilder<EncryptMode<R, D>, P>
where
    R: RngCore + CryptoRng,
    D: AsRef<[u8]>,
    P: AsRef<std::path::Path>,
{
    pub fn new_encrypt(
        dir_path: P,
        password: impl AsRef<[u8]>,
        data: D,
        rng: R,
        algorithm: KdfAlgorithm,
        file_name: &str,
    ) -> Self {
        Self {
            path: dir_path,
            password: password.as_ref().to_vec(),
            crypto_mode: EncryptMode {
                rng,
                algorithm,
                data,
                file_name: file_name.to_string(),
            },
        }
    }

    pub fn save(mut self) -> Result<(), crate::Error> {
        self.process_encryption()?;
        // let file_name = self.generate_filename()?; // 生成唯一文件名
        // std::fs::write(self.path.join(&file_name), encrypted)?;
        Ok(())
    }

    fn process_encryption(&mut self) -> Result<(), crate::Error>
    where
        D: AsRef<[u8]>,
        R: rand::Rng + rand::CryptoRng,
        P: AsRef<std::path::Path>,
    {
        // let data_bytes = self.data.to_bytes()?;
        let salt = crate::generate_random_bytes(&mut self.crypto_mode.rng, DEFAULT_KEY_SIZE);
        let kdf = KdfFactory::create(&self.crypto_mode.algorithm, &salt)?;
        let engine = KeystoreEngine::new(kdf);

        let file_path = self.path.as_ref().join(&self.crypto_mode.file_name);
        let _keystore = KeystoreFile::new(file_path, engine).save(
            &mut self.crypto_mode.rng,
            &self.crypto_mode.data,
            self.password.as_ref(),
        )?;

        // let data = wallet_utils::conversion::vec_to_string(self.data.as_ref())?;
        // engine.encrypt(self.crypto_mode.rng, &data_bytes, &self.password)
        Ok(())
    }
}

impl<P> KeystoreBuilder<DecryptMode, P>
where
    P: AsRef<std::path::Path>,
{
    pub fn new_decrypt(path: P, password: impl AsRef<[u8]>) -> Self {
        Self {
            path,
            password: password.as_ref().to_vec(),
            crypto_mode: DecryptMode {},
        }
    }

    // pub fn load<D>(self) -> Result<D, crate::Error>
    pub fn load(self) -> Result<RecoverableData, crate::Error>
// where
    //     D: serde::de::DeserializeOwned,
    {
        let mut contents = String::new();
        wallet_utils::file_func::read(&mut contents, self.path.as_ref())?;

        self.process_decryption(&contents)
    }

    /// 解密处理
    // fn process_decryption<D>(&self, encrypted: &str) -> Result<D, crate::Error>
    fn process_decryption(&self, encrypted: &str) -> Result<RecoverableData, crate::Error>
// where
    //     D: serde::de::DeserializeOwned,
    {
        // let engine = KeystoreEngine::from_file(self.path.join(&self.crypto_mode.file_name))?;
        let keystore: KeystoreJson = wallet_utils::serde_func::serde_from_str(encrypted)?;

        let kdf = KdfFactory::create_from_file(&keystore)?;
        let engine = KeystoreEngine::new(kdf);

        let decrypted = engine.decrypt(self.password.as_ref(), keystore)?;

        // Ok(wallet_utils::conversion::vec_to_string(&decrypted)?)
        Ok(RecoverableData(decrypted))
        // D::from_bytes(&decrypted)
    }
}

// impl<T> KeystoreData for T
// where
//     T: serde::Serialize + AsRef<[u8]>,
// {
//     fn to_bytes(&self) -> Result<Vec<u8>, crate::Error> {
//         Ok(wallet_utils::serde_func::serde_to_vec(self)?)
//     }
// }

pub struct RecoverableData(Vec<u8>);

impl RecoverableData {
    pub fn into_string(self) -> Result<String, crate::Error> {
        Ok(wallet_utils::conversion::vec_to_string(&self.0)?)
    }

    pub(crate) fn inner(self) -> Vec<u8> {
        self.0
    }
}

// impl<T> RecoverableData for T
// where
//     T: serde::de::DeserializeOwned,
// {
//     fn from_bytes(data: &[u8]) -> Result<Self, crate::Error> {
//         let str = wallet_utils::conversion::vec_to_string(data)?;
//         tracing::info!("str: {str:?}");
//         Ok(wallet_utils::serde_func::serde_from_str(&str)?)
//     }
// }

// pub trait KeystoreData: AsRef<[u8]> {
//     fn to_bytes(&self) -> Result<Vec<u8>, crate::Error>;
// }

// pub trait RecoverableData: Sized {
//     fn from_bytes(data: &[u8]) -> Result<Self, crate::Error>;
// }
