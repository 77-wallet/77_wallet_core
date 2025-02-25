use std::path::Path;

use crate::error::crypto::KeystoreError;

use super::KeystoreJson;

pub trait KeystoreIo {
    fn save(&self, keystore: &KeystoreJson, path: &Path) -> Result<(), KeystoreError>;
    fn load(&self, path: &Path) -> Result<KeystoreJson, KeystoreError>;
}

pub struct JsonFileIo;

impl KeystoreIo for JsonFileIo {
    fn save(&self, keystore: &KeystoreJson, path: &Path) -> Result<(), KeystoreError> {
        let contents = serde_json::to_string(keystore)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    fn load(&self, path: &Path) -> Result<KeystoreJson, KeystoreError> {
        let contents = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&contents)?)
    }
}
