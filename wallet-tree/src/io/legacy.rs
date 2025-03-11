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

    fn load(
        &self,
        naming: Box<dyn crate::naming::NamingStrategy>,
        account_index_map: Option<&wallet_utils::address::AccountIndexMap>,
        address: &str,
        chain_code: &str,
        derivation_path: &str,
        subs_dir: &dyn AsRef<std::path::Path>,
        password: &str,
        // ) -> Result<Box<dyn TryFrom<RecoverableData>>, crate::Error>;
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
        // let pk: PkWallet = data.try_into()?;
        let pk: PkWallet = data.try_into()?;

        Ok(pk.pkey())
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

    // fn store(
    //     &self,
    //     name: &str,
    //     data: D,
    //     file_path: &P,
    //     password: &str,
    //     algorithm: wallet_keystore::KdfAlgorithm,
    // ) -> Result<(), crate::Error> {
    //     let rng = rand::thread_rng();
    //     KeystoreBuilder::new_encrypt(file_path, password, data, rng, algorithm, &name).save()?;
    //     Ok(())
    // }

    // fn load(&self, path: P, password: &str) -> Result<D, crate::Error>
    // where
    //     P: AsRef<std::path::Path>,
    //     D: TryFrom<wallet_keystore::RecoverableData> + Sized,
    //     crate::Error: From<<D as TryFrom<wallet_keystore::RecoverableData>>::Error>,
    // {
    //     let data = KeystoreBuilder::new_decrypt(path, password).load()?;
    //     Ok(D::try_from(data)?)
    // }
}
