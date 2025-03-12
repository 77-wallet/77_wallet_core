use crate::{
    error::crypto::KeystoreError,
    keystore::factory::{Argon2idParams, KdfParams},
    KdfAlgorithm,
};

use super::KeyDerivation;

pub struct Argon2idKdf {
    pub params: Argon2idParams,
}

impl Argon2idKdf {
    pub fn new(params: Argon2idParams) -> Self {
        Self { params }
    }

    /// 推荐的安全参数（根据 OWASP 2023 建议）
    pub fn recommended_params<R: rand::RngCore>(rng: &mut R) -> Self {
        let salt = {
            let mut salt = vec![0u8; 16];
            rng.fill_bytes(&mut salt);
            salt
        };

        Self {
            params: Argon2idParams {
                dklen: 32,          // 32字节密钥
                time_cost: 3,       // 3次迭代
                memory_cost: 65536, // 64MB (单位是KB)
                parallelism: 1,
                salt,
            },
        }
    }
}

impl KeyDerivation for Argon2idKdf {
    fn derive_key(&self, password: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        let start = std::time::Instant::now();
        let params = argon2::Params::new(
            self.params.memory_cost,
            self.params.time_cost,
            self.params.parallelism,
            None,
        )
        .map_err(KeystoreError::Argon2)?;

        let argon2 =
            argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
        // argon2.hash_password
        // argon2.verify_password(b"password", hash);
        let mut output_key = vec![0u8; self.params.dklen as usize];
        argon2
            .hash_password_into(password, &self.params.salt, &mut output_key)
            .map_err(KeystoreError::Argon2)?;
        tracing::info!("Argon2id KDF took {}ms", start.elapsed().as_millis());
        Ok(output_key)
    }

    fn params(&self) -> KdfParams {
        KdfParams::Argon2id(self.params.clone())
    }

    fn algorithm(&self) -> KdfAlgorithm {
        KdfAlgorithm::Argon2id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn test_key_derivation() {
        let mut rng = OsRng;
        let kdf = Argon2idKdf::recommended_params(&mut rng);
        let password = b"my_strong_password";

        // 测试正常派生
        let key1 = kdf.derive_key(password).unwrap();
        assert_eq!(key1.len(), 32);

        // 测试相同密码生成相同密钥（相同盐）
        let key2 = kdf.derive_key(password).unwrap();
        assert_eq!(key1, key2);

        // 测试不同盐生成不同密钥
        let mut different_salt_params = kdf.params.clone();
        different_salt_params.salt.iter_mut().for_each(|b| *b = !*b);
        let kdf2 = Argon2idKdf::new(different_salt_params);
        let key3 = kdf2.derive_key(password).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_invalid_params() {
        let invalid_params = Argon2idParams {
            dklen: 32,
            time_cost: 0, // 无效参数
            memory_cost: 8,
            parallelism: 1,
            salt: vec![0; 16],
        };

        let kdf = Argon2idKdf::new(invalid_params);
        let result = kdf.derive_key(b"password");
        assert!(matches!(result, Err(KeystoreError::Argon2(_))));
    }
}
