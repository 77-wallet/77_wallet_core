use serde::{Deserialize, Serialize};

use crate::crypto::{
    KdfParams, KeyDerivationFunction,
    argon2id::Argon2idKdf,
    pbkdf2::Pbkdf2Kdf,
    scrypt_::{ScryptKdf, ScryptParams},
};

use super::json::KeystoreJson;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Types of key derivition functions supported by the Web3 Secret Storage.
pub enum KdfAlgorithm {
    Pbkdf2,
    Scrypt,
    Argon2id,
}

pub struct KdfFactory;

impl KdfFactory {
    pub fn create(
        algorithm: &KdfAlgorithm,
        salt: &[u8],
    ) -> Result<Box<dyn KeyDerivationFunction>, crate::Error> {
        match algorithm {
            KdfAlgorithm::Scrypt => {
                let params = ScryptParams::default().with_salt(salt);
                Ok(Box::new(ScryptKdf::new(params)))
            }
            KdfAlgorithm::Pbkdf2 => {
                todo!()
            }
            KdfAlgorithm::Argon2id => {
                let params = Argon2idKdf::recommended_params_with_salt(salt);
                Ok(Box::new(params))
            }
        }
    }

    pub fn create_from_file(
        keystore: &KeystoreJson,
    ) -> Result<Box<dyn KeyDerivationFunction>, crate::Error> {
        let kdf: Box<dyn KeyDerivationFunction> = match &keystore.crypto.kdfparams {
            KdfParams::Pbkdf2(p) => Box::new(Pbkdf2Kdf::new(p.to_owned())),
            KdfParams::Scrypt(p) => Box::new(ScryptKdf::new(p.to_owned())),
            KdfParams::Argon2id(p) => Box::new(Argon2idKdf::new(p.to_owned())),
        };

        Ok(kdf)
    }
}
