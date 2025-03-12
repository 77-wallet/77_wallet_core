use serde::Serialize;
use wallet_keystore::{wallet::prikey::PkWallet, KeystoreBuilder};

use crate::naming::FileType;

use super::IoStrategy;

#[derive(Debug, Default, PartialEq, Clone, Serialize)]
pub struct LegacyIo;

impl IoStrategy for LegacyIo {
    fn store(
        &self,
        name: &str,
        data: &dyn AsRef<[u8]>,
        file_path: &dyn AsRef<std::path::Path>,
        password: &str,
        algorithm: wallet_keystore::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let rng = rand::thread_rng();
        KeystoreBuilder::new_encrypt(file_path, password, data, rng, algorithm, &name).save()?;
        Ok(())
    }

    fn load_root(
        &self,
        naming: Box<dyn crate::naming::NamingStrategy>,
        wallet_address: &str,
        root_dir: &dyn AsRef<std::path::Path>,
        password: &str,
    ) -> Result<super::RootData, crate::Error> {
        let seed_meta =
            naming.generate_filemeta(FileType::Seed, &wallet_address, None, None, None)?;
        let seed_filename = naming.encode(seed_meta)?;

        let seed = crate::Keystore::load_data::<_, wallet_keystore::wallet::seed::SeedWallet>(
            root_dir.as_ref().join(seed_filename),
            password,
        )?;

        let phrase_meta =
            naming.generate_filemeta(FileType::Phrase, &wallet_address, None, None, None)?;
        let phrase_filename = naming.encode(phrase_meta)?;

        let phrase_wallet = crate::Keystore::load_data::<
            _,
            wallet_keystore::wallet::phrase::PhraseWallet,
        >(root_dir.as_ref().join(phrase_filename), password)?;

        Ok(super::RootData::new(phrase_wallet.phrase, seed.seed))
    }

    fn load_subkey(
        &self,
        naming: Box<dyn crate::naming::NamingStrategy>,
        account_index_map: Option<&wallet_utils::address::AccountIndexMap>,
        address: &str,
        chain_code: &str,
        derivation_path: &str,
        subs_dir: &dyn AsRef<std::path::Path>,
        password: &str,
    ) -> Result<Vec<u8>, crate::Error> {
        let pk_meta = naming.generate_filemeta(
            FileType::DerivedData,
            &address,
            account_index_map,
            Some(chain_code.to_string()),
            Some(derivation_path.to_string()),
        )?;
        let pk_filename = naming.encode(pk_meta)?;
        let data =
            KeystoreBuilder::new_decrypt(subs_dir.as_ref().join(pk_filename), password).load()?;
        let pk: PkWallet = data.try_into()?;

        Ok(pk.pkey())
    }

    fn store_root(
        &self,
        naming: Box<dyn crate::naming::NamingStrategy>,
        address: &str,
        seed: &[u8],
        phrase: &str,
        file_path: &dyn AsRef<std::path::Path>,
        password: &str,
        algorithm: wallet_keystore::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let phrase_meta = naming.generate_filemeta(FileType::Phrase, &address, None, None, None)?;
        let seed_meta = naming.generate_filemeta(FileType::Seed, &address, None, None, None)?;
        let seed_filename = naming.encode(seed_meta)?;
        let phrase_filename = naming.encode(phrase_meta)?;

        crate::Keystore::store_data(
            &seed_filename,
            seed,
            &file_path,
            password,
            algorithm.clone(),
        )?;
        crate::Keystore::store_data(&phrase_filename, phrase, &file_path, password, algorithm)?;
        Ok(())
    }

    fn store_subkey(
        &self,
        naming: Box<dyn crate::naming::NamingStrategy>,
        account_index_map: &wallet_utils::address::AccountIndexMap,
        address: &str,
        chain_code: &str,
        derivation_path: &str,
        data: &dyn AsRef<[u8]>,
        file_path: &dyn AsRef<std::path::Path>,
        password: &str,
        algorithm: wallet_keystore::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let file_meta = naming.generate_filemeta(
            FileType::DerivedData,
            &address,
            Some(account_index_map),
            Some(chain_code.to_string()),
            Some(derivation_path.to_string()),
        )?;

        let name = naming.encode(file_meta)?;

        let rng = rand::thread_rng();
        KeystoreBuilder::new_encrypt(file_path, password, data, rng, algorithm, &name).save()?;
        Ok(())
    }

    fn store_subkeys_bulk(
        &self,
        naming: Box<dyn crate::naming::NamingStrategy>,
        subkeys: Vec<super::BulkSubkey>,
        file_path: &dyn AsRef<std::path::Path>,
        password: &str,
        algorithm: wallet_keystore::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        for subkey in subkeys {
            let file_meta = naming.generate_filemeta(
                FileType::DerivedData,
                &subkey.address,
                Some(&subkey.account_index_map),
                Some(subkey.chain_code.to_string()),
                Some(subkey.derivation_path.to_string()),
            )?;

            let name = naming.encode(file_meta)?;
            let rng = rand::thread_rng();
            KeystoreBuilder::new_encrypt(
                file_path,
                password,
                &subkey.data,
                rng,
                algorithm.clone(),
                &name,
            )
            .save()?;
        }
        Ok(())
    }

    fn delete_root(
        &self,
        naming: Box<dyn crate::naming::NamingStrategy>,
        address: &str,
        root_dir: &dyn AsRef<std::path::Path>,
    ) -> Result<(), crate::Error> {
        let path = root_dir
            .as_ref()
            .join(naming.encode(naming.generate_filemeta(
                FileType::PrivateKey,
                &address,
                None,
                None,
                None,
            )?)?);
        wallet_utils::file_func::remove_file(path)?;
        let path = root_dir
            .as_ref()
            .join(naming.encode(naming.generate_filemeta(
                FileType::Phrase,
                &address,
                None,
                None,
                None,
            )?)?);
        wallet_utils::file_func::remove_file(path)?;
        let path = root_dir
            .as_ref()
            .join(naming.encode(naming.generate_filemeta(
                FileType::Seed,
                &address,
                None,
                None,
                None,
            )?)?);
        wallet_utils::file_func::remove_file(path)?;

        Ok(())
    }
}
