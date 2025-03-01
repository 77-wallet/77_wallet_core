use std::path::Path;

use crate::keystore::builder::{KeystoreBuilder, RecoverableData};

#[derive(Debug, Clone, Default)]
pub struct Keystore {}

impl Keystore {
    pub fn store_data<D: AsRef<[u8]>, P: AsRef<Path>>(
        name: &str,
        data: D,
        file_path: &P,
        password: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let rng = rand::thread_rng();
        KeystoreBuilder::new_encrypt(file_path, password, data, rng, algorithm, &name).save()?;

        Ok(())
    }

    pub(crate) fn load_data<P, D>(path: P, password: &str) -> Result<D, crate::Error>
    where
        P: AsRef<Path>,
        D: TryFrom<RecoverableData> + Sized,
        crate::Error: From<<D as TryFrom<RecoverableData>>::Error>,
    {
        let data = KeystoreBuilder::new_decrypt(path, password).load()?;
        Ok(D::try_from(data)?)
    }

    pub fn load_phrase_keystore<P: AsRef<Path>>(
        address: &str,
        directory: &P,
        password: &str,
    ) -> Result<crate::wallet::phrase::PhraseWallet, crate::Error> {
        let name = wallet_tree::wallet_tree::root::RootKeystoreInfo::new(
            wallet_tree::utils::file::Suffix::phrase(),
            address,
        )
        .gen_name_with_address()?;
        let path = directory.as_ref().join(name);

        let phrase = KeystoreBuilder::new_decrypt(path, password).load()?;
        Ok(crate::wallet::phrase::PhraseWallet::from_phrase(
            &phrase.into_string()?,
        )?)
    }

    pub fn load_seed_keystore<P: AsRef<Path>>(
        address: &str,
        directory: P,
        password: &str,
    ) -> Result<crate::wallet::seed::SeedWallet, crate::Error> {
        let name = wallet_tree::wallet_tree::root::RootKeystoreInfo::new(
            wallet_tree::utils::file::Suffix::seed(),
            address,
        )
        .gen_name_with_address()?;
        let path = directory.as_ref().join(name);

        let phrase = KeystoreBuilder::new_decrypt(path, password).load()?;
        Ok(crate::wallet::seed::SeedWallet::from_seed(phrase.inner())?)
    }
}
