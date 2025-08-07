use rand::{CryptoRng, RngCore};

use crate::{
    EncryptedJson, KdfAlgorithm,
    crypto::encrypted_json::cryptor::{EncryptedJsonDecryptor, EncryptedJsonGenerator},
    keystore::generator::{KeystoreJsonDecryptor, KeystoreJsonGenerator},
};

use super::file::KeystoreFile;

pub struct KeystoreBuilder<M, P: AsRef<std::path::Path>> {
    path: P,
    password: Vec<u8>,
    crypto_mode: M,
}

pub struct EncryptMode<R: Clone, D> {
    rng: R,
    algorithm: KdfAlgorithm,
    file_name: String,
    data: D,
}

pub struct DecryptMode {}

impl<R, D, P> KeystoreBuilder<EncryptMode<R, D>, P>
where
    R: RngCore + CryptoRng + Clone,
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
        let data = self.process_encryption()?;
        let data = wallet_utils::serde_func::serde_to_string(&data)?;

        let file_path = self.path.as_ref().join(&self.crypto_mode.file_name);
        KeystoreFile::new(file_path).save(&data)?;
        Ok(())
    }

    fn process_encryption(&mut self) -> Result<EncryptedJson, crate::Error>
    where
        D: AsRef<[u8]>,
        R: rand::Rng + rand::CryptoRng,
        P: AsRef<std::path::Path>,
    {
        KeystoreJsonGenerator::new(
            self.crypto_mode.rng.clone(),
            self.crypto_mode.algorithm.clone(),
        )
        .generate(&self.password, &self.crypto_mode.data.as_ref())
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

    pub fn load(self) -> Result<RecoverableData, crate::Error> {
        let mut contents = String::new();
        wallet_utils::file_func::read(&mut contents, self.path.as_ref())?;
        self.process_decryption(&contents)
    }

    /// 解密处理
    fn process_decryption(&self, encrypted: &str) -> Result<RecoverableData, crate::Error> {
        let decrypted = KeystoreJsonDecryptor.decrypt(&self.password, encrypted)?;
        Ok(RecoverableData(decrypted))
    }
}

#[derive(Debug)]
pub struct RecoverableData(Vec<u8>);

impl RecoverableData {
    pub fn into_string(self) -> Result<String, crate::Error> {
        Ok(wallet_utils::conversion::vec_to_string(&self.0)?)
    }

    pub fn inner(self) -> Vec<u8> {
        self.0
    }
}
