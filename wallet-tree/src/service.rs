use std::path::Path;

use wallet_keystore::{KdfAlgorithm, KeystoreBuilder, RecoverableData};

#[derive(Debug, Clone, Default)]
pub struct Keystore {}

impl Keystore {
    pub fn store_data<D: AsRef<[u8]>, P: AsRef<Path>>(
        name: &str,
        data: D,
        file_path: &P,
        password: &str,
        algorithm: KdfAlgorithm,
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
}
