use crate::error::crypto::KeystoreError;

use super::{
    kdfs::{argon2id::Argon2idKdf, pbkdf2::Pbkdf2Kdf, scrypt_::ScryptKdf, KeyDerivationFunction},
    KdfParams,
};
/// 密钥派生上下文
/// 管理密钥派生策略
pub struct KdfContext {
    // params: KdfParams,
    strategy: Box<dyn KeyDerivationFunction>,
}

impl KdfContext {
    pub fn new(params: KdfParams) -> Result<Self, crate::Error> {
        let strategy: Box<dyn KeyDerivationFunction> = match &params {
            KdfParams::Pbkdf2(p) => Box::new(Pbkdf2Kdf::new(p.to_owned())),
            KdfParams::Scrypt(p) => Box::new(ScryptKdf::new(p.to_owned())),
            KdfParams::Argon2id(p) => Box::new(Argon2idKdf::new(p.to_owned())),
        };

        Ok(Self { strategy })
    }

    pub fn derive_key(&self, password: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        // let salt = match &self.params {
        //     KdfParams::Pbkdf2(pbkdf2_params) => pbkdf2_params.salt.as_slice(),
        //     KdfParams::Scrypt(scrypt_params) => scrypt_params.salt.as_slice(),
        //     KdfParams::Argon2id(argon2id_params) => argon2id_params.salt.as_slice(),
        // };
        self.strategy.derive_key(password)
    }
}
