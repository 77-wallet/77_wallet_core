use crate::{error::crypto::KeystoreError, keystore::factory::KdfParams, KdfAlgorithm};

pub(crate) mod argon2id;
pub(crate) mod pbkdf2;
pub(crate) mod scrypt_;

pub trait KeyDerivationFunction {
    fn derive_key(&self, password: &[u8]) -> Result<Vec<u8>, KeystoreError>;

    fn params(&self) -> KdfParams;

    fn algorithm(&self) -> KdfAlgorithm;
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
    use crate::{
        crypto::kdfs::{argon2id::Argon2idKdf, log2, KeyDerivationFunction as _},
        generate_random_bytes,
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
        // let salt = generate_random_bytes(&mut rng, 16);

        // let salt = "somesalt".as_bytes();
        let start = std::time::Instant::now();
        let key1 = kdf.derive_key(b"password").unwrap();
        println!("time: {}", start.elapsed().as_millis());
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
