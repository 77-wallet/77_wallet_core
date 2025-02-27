use crate::{
    error::crypto::KeystoreError,
    keystore::factory::{Argon2idParams, KdfParams, Pbkdf2Params, ScryptParams},
};
use hmac::Hmac;
use scrypt::{scrypt, Params as ScryptParams_};
use sha2::Sha256;

pub trait KeyDerivation {
    fn derive_key(&self, password: &[u8], salt: &[u8]) -> Result<Vec<u8>, KeystoreError>;

    fn params(&self) -> KdfParams;
}
pub struct ScryptKdf {
    pub params: ScryptParams,
}

impl ScryptKdf {
    pub fn new(params: ScryptParams) -> Self {
        Self { params }
    }
}

impl KeyDerivation for ScryptKdf {
    fn derive_key(&self, password: &[u8], salt: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        let mut key = vec![0u8; self.params.dklen as usize];
        let log_n = super::kdf::log2(self.params.n) as u8;
        let scrypt_params = ScryptParams_::new(log_n, self.params.r, self.params.p)?;
        scrypt(password, salt, &scrypt_params, &mut key)?;
        Ok(key)
    }

    fn params(&self) -> KdfParams {
        KdfParams::Scrypt(self.params.clone())
    }
}

pub struct Pbkdf2Kdf {
    pub params: Pbkdf2Params,
}

impl Pbkdf2Kdf {
    pub fn new(params: Pbkdf2Params) -> Self {
        Self { params }
    }
}

impl KeyDerivation for Pbkdf2Kdf {
    fn derive_key(&self, password: &[u8], salt: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        let mut key = vec![0u8; self.params.dklen as usize];
        pbkdf2::pbkdf2::<Hmac<Sha256>>(password, salt, self.params.c, &mut key);
        Ok(key)
    }

    fn params(&self) -> KdfParams {
        KdfParams::Pbkdf2(self.params.clone())
    }
}

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
    fn derive_key(&self, password: &[u8], salt: &[u8]) -> Result<Vec<u8>, KeystoreError> {
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
            .hash_password_into(password, salt, &mut output_key)
            .map_err(KeystoreError::Argon2)?;
        Ok(output_key)
    }

    fn params(&self) -> KdfParams {
        KdfParams::Argon2id(self.params.clone())
    }
}

#[allow(unused_assignments)]
pub(crate) fn log2(mut n: u32) -> u32 {
    let mut result = 0;
    if (n & 0xffff0000) != 0 {
        result += 16;
        n >>= 16;
    }
    if (n & 0x0000ff00) != 0 {
        result += 8;
        n >>= 8;
    }
    if (n & 0x000000f0) != 0 {
        result += 4;
        n >>= 4;
    }
    if (n & 0x0000000c) != 0 {
        result += 2;
        n >>= 2;
    }
    if (n & 0x00000002) != 0 {
        result += 1;
        n >>= 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::crypto::{
        generate_random_bytes,
        kdf::{log2, Argon2idKdf, KeyDerivation as _},
    };

    #[test]
    fn test_log2() {
        let n = 8192;
        println!("log_n: {}", log2(n));
    }

    #[test]
    fn test_argon2id_basic() {
        let mut rng = rand::thread_rng();
        let kdf = Argon2idKdf::recommended_params(&mut rng);
        let salt = generate_random_bytes(&mut rng, 16);

        let salt = "somesalt".as_bytes();

        let key1 = kdf.derive_key(b"password", &salt).unwrap();
        let encode = hex::encode(key1);
        println!("encode: {}", encode);
        // let key2 = kdf.derive_key(b"password", &salt).unwrap();

        // assert_eq!(key1, key2);
        assert_eq!(
            "9e8789c8b42834220afc00085ac73acc308651216994abbfddd69b2592032efd",
            encode
        );
    }

    // #[test]
    // fn test_argon2id_param_validation() {
    //     let invalid_params = Argon2idParams {
    //         memory_cost: 0,
    //         dklen: todo!(),
    //         time_cost: todo!(),
    //         parallelism: todo!(),
    //         salt: todo!(), // 非法值
    //                         // ...其他参数
    //     };

    //     let result = Argon2idKdf::new(invalid_params).derive_key(b"", &[]);
    //     assert!(matches!(result, Err(KeystoreError::Argon2Error(_))));
    // }
}
