use std::path::Path;

use crate::keystore::builder::KeystoreBuilder;

#[derive(Debug, Clone, Default)]
pub struct Keystore {}

impl Keystore {
    pub fn store_root_private_key<P: AsRef<Path>>(
        // address: &str,
        name: &str,
        private_key: &[u8],
        file_path: &P,
        password: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let rng = rand::thread_rng();

        KeystoreBuilder::new_encrypt(file_path, password, private_key, rng, algorithm, &name)
            .save()?;

        Ok(())
    }

    pub fn store_seed_keystore<P: AsRef<Path>>(
        // address: &str,
        name: &str,
        seed: &[u8],
        directory: &P,
        password: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let rng = rand::thread_rng();

        KeystoreBuilder::new_encrypt(directory, password, seed, rng, algorithm, &name).save()?;

        Ok(())
    }

    pub fn store_phrase_keystore<P: AsRef<Path>>(
        // address: &str,
        name: &str,
        phrase: &str,
        directory: &P,
        password: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let rng = rand::thread_rng();
        KeystoreBuilder::new_encrypt(directory, password, phrase, rng, algorithm, &name).save()?;
        Ok(())
    }

    pub fn store_sub_private_key<P: AsRef<Path>>(
        address_generator: Box<
            dyn wallet_core::address::GenAddress<
                Address = wallet_chain_instance::instance::Address,
                Error = wallet_chain_instance::Error,
            >,
        >,
        private_key: Vec<u8>,
        file_path: P,
        password: &str,
        address: &str,
        derivation_path: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let rng = rand::thread_rng();

        let name = wallet_tree::wallet_tree::subs::SubsKeystoreInfo::new(
            derivation_path,
            wallet_tree::utils::file::Suffix::pk(),
            address_generator.chain_code(),
            address,
        )
        .gen_name_with_derivation_path()?;

        KeystoreBuilder::new_encrypt(file_path, password, private_key, rng, algorithm, &name)
            .save()?;

        Ok(())
    }

    pub fn load_private_key_keystore<P: AsRef<Path>>(
        file_path: P,
        password: &str,
    ) -> Result<crate::wallet::prikey::PkWallet, crate::Error> {
        let phrase = KeystoreBuilder::new_decrypt(file_path, password).load()?;
        Ok(crate::wallet::prikey::PkWallet::from_pkey(&phrase.inner())?)
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
