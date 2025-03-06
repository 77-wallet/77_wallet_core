pub(crate) mod legacy;
pub(crate) mod modern;

use std::path::Path;

use wallet_keystore::{KdfAlgorithm, KeystoreBuilder, RecoverableData};

use crate::naming::NamingStrategy;

pub trait IoStrategy: Send + Sync {
    fn store(
        &self,
        name: &str,
        data: &dyn AsRef<[u8]>,
        file_path: &dyn AsRef<Path>,
        password: &str,
        algorithm: KdfAlgorithm,
    ) -> Result<(), crate::Error>;

    fn load_custom(
        &self,
        subs_dir: &dyn AsRef<Path>,
        name: &str,
        password: &str,
    ) -> Result<RecoverableData, crate::Error> {
        let data = KeystoreBuilder::new_decrypt(subs_dir.as_ref().join(name), password).load()?;

        Ok(data)
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
    ) -> Result<Vec<u8>, crate::Error>;
    // where
    //     P: AsRef<Path>,
    //     D: TryFrom<RecoverableData> + Sized,
    //     crate::Error: From<<D as TryFrom<RecoverableData>>::Error>;

    // fn load_subkey(
    //     &self,
    //     naming: Box<dyn crate::naming::NamingStrategy>,
    //     account_index_map: &wallet_utils::address::AccountIndexMap,
    //     address: &str,
    //     chain_code: &str,
    //     derivation_path: &str,
    //     path: &dyn AsRef<std::path::Path>,
    //     password: &str,
    // ) -> Result<RecoverableData, crate::Error>;

    fn store_subkey(
        &self,
        naming: Box<dyn NamingStrategy>,
        account_index_map: &wallet_utils::address::AccountIndexMap,
        address: &str,
        chain_code: &str,
        derivation_path: &str,
        data: &dyn AsRef<[u8]>,
        file_path: &dyn AsRef<Path>,
        password: &str,
        algorithm: KdfAlgorithm,
    ) -> Result<(), crate::Error>;
}
