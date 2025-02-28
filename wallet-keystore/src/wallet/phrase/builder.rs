use crate::keystore::{
    engine::KeystoreEngine, factory::KdfFactory, file::KeystoreFile, json::KeystoreJson,
};

use super::PhraseWallet;

const DEFAULT_KEY_SIZE: usize = 32usize;

pub(crate) struct PhraseEncryptorBuilder<'a, P, R, B, S> {
    keypath: P,
    rng: &'a mut R,
    data: B,
    password: S,
    name: &'a str,
    algorithm: crate::keystore::factory::KdfAlgorithm,
}

impl<'a, P, R, B, S> PhraseEncryptorBuilder<'a, P, R, B, S>
where
    P: AsRef<std::path::Path>,
    R: rand::Rng + rand::CryptoRng,
    B: AsRef<[u8]>,
    S: AsRef<[u8]>,
{
    pub(crate) fn new(
        keypath: P,
        rng: &'a mut R,
        data: B,
        password: S,
        name: &'a str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Self {
        PhraseEncryptorBuilder {
            keypath,
            rng,
            data,
            password,
            name,
            algorithm,
        }
    }
}

// impl<P, R, B, S> crate::wallet::WalletEncrypt for PhraseEncryptorBuilder<'_, P, R, B, S>
// where
//     P: AsRef<std::path::Path>,
//     R: rand::Rng + rand::CryptoRng,
//     B: AsRef<[u8]>,
//     S: AsRef<[u8]>,
// {
//     type Output = PhraseWallet;

//     fn encrypt_keystore(self) -> Result<Self::Output, crate::Error> {
//         let data = self.data.as_ref();
//         let uuid = crate::crypto::encrypt_data(
//             self.keypath,
//             self.rng,
//             data,
//             self.password,
//             Some(self.name),
//             self.algorithm,
//         )?;
//         let data = wallet_utils::conversion::vec_to_string(data)?;
//         Ok(PhraseWallet::from_phrase(&data)?)
//     }
// }

impl<P, R, B, S> crate::wallet::WalletEncrypt for PhraseEncryptorBuilder<'_, P, R, B, S>
where
    P: AsRef<std::path::Path>,
    R: rand::Rng + rand::CryptoRng,
    B: AsRef<[u8]>,
    // B: serde::Serialize,
    S: AsRef<[u8]>,
{
    type Output = PhraseWallet;

    fn encrypt_keystore(self) -> Result<Self::Output, crate::Error> {
        // let data = self.data.as_ref();
        let dir = self.keypath.as_ref().join(self.name);
        let salt = crate::generate_random_bytes(self.rng, DEFAULT_KEY_SIZE);
        let kdf = KdfFactory::create(&self.algorithm, &salt)?;
        let engine = KeystoreEngine::new(kdf);

        let _keystore =
            KeystoreFile::new(dir, engine).save(self.rng, &self.data, self.password.as_ref())?;
        // let uuid = crate::crypto::encrypt_data(
        //     self.keypath,
        //     self.rng,
        //     data,
        //     self.password,
        //     self.name,
        //     self.algorithm,
        // )?;
        let data = wallet_utils::conversion::vec_to_string(self.data.as_ref())?;
        // let data = wallet_utils::serde_func::serde_to_string(&self.data)?;
        Ok(PhraseWallet::from_phrase(&data)?)
    }
}

pub(crate) struct PhraseDecryptorBuilder<P, S> {
    keypath: P,
    password: S,
}

impl<P, S> PhraseDecryptorBuilder<P, S>
where
    P: AsRef<std::path::Path>,
    S: AsRef<[u8]>,
{
    pub(crate) fn new(keypath: P, password: S) -> Self {
        PhraseDecryptorBuilder { keypath, password }
    }
}

impl<'a, P, S> crate::wallet::WalletDecrypt for PhraseDecryptorBuilder<P, S>
where
    P: AsRef<std::path::Path>,
    S: AsRef<[u8]>,
{
    type Output = PhraseWallet;

    fn decrypt_keystore(self) -> Result<Self::Output, crate::Error> {
        let mut contents = String::new();
        wallet_utils::file_func::read(&mut contents, &self.keypath)?;
        let keystore: KeystoreJson = wallet_utils::serde_func::serde_from_str(&contents)?;

        let kdf = KdfFactory::create_from_file(&keystore)?;
        let engine = KeystoreEngine::new(kdf);

        let phrase = engine.decrypt(self.password.as_ref(), keystore)?;
        // let phrase = crate::crypto::decrypt_data(self.keypath, self.password)?;
        let phrase = wallet_utils::conversion::vec_to_string(&phrase)?;
        Ok(PhraseWallet::from_phrase(&phrase)?)
    }
}
