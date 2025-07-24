#![cfg_attr(docsrs, feature(doc_cfg))]
//! A minimalist library to interact with encrypted JSON keystores as per the
//! [Web3 Secret Storage Definition](https://github.com/ethereum/wiki/wiki/Web3-Secret-Storage-Definition).

pub mod argon2id;
pub mod pbkdf2;
pub mod scrypt_;

use argon2id::Argon2idParams;
use pbkdf2::Pbkdf2Params;
use scrypt_::ScryptParams;

use crate::{KdfAlgorithm, error::crypto::KeystoreError};

pub trait KeyDerivationFunction {
    fn derive_key(&self, password: &[u8]) -> Result<Vec<u8>, KeystoreError>;

    fn params(&self) -> KdfParams;

    fn algorithm(&self) -> KdfAlgorithm;
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum KdfParams {
    Pbkdf2(Pbkdf2Params),
    Scrypt(ScryptParams),
    Argon2id(Argon2idParams),
}

impl KdfParams {
    pub fn algorithm(&self) -> KdfAlgorithm {
        match self {
            Self::Pbkdf2(_) => KdfAlgorithm::Pbkdf2,
            Self::Scrypt(_) => KdfAlgorithm::Scrypt,
            Self::Argon2id(_) => KdfAlgorithm::Argon2id,
        }
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
mod test {

    use crate::keystore::{
        cipher::{self, SymmetricCipher as _},
        json::{CipherparamsJson, CryptoJson, KeystoreJson},
        mac::{self, MacCalculator as _},
    };
    use rand::{CryptoRng, Rng};
    use scrypt::scrypt;
    use tempfile::tempdir;
    use uuid::Uuid;

    use std::{fs::File, io::Write, path::Path};

    use crate::{error::crypto::KeystoreError, keystore::factory::KdfFactory};

    const DEFAULT_CIPHER: &str = "aes-128-ctr";
    const DEFAULT_KEY_SIZE: usize = 32usize;
    const DEFAULT_IV_SIZE: usize = 16usize;

    const TEST_PASSWORD: &str = "test_password";
    const TEST_PLAINTEXT: &[u8] = b"this is a test plaintext";

    use hex::FromHex;

    use crate::KdfAlgorithm;

    use super::{
        KdfParams, KeyDerivationFunction, argon2id::Argon2idKdf, pbkdf2::Pbkdf2Kdf,
        scrypt_::ScryptKdf,
    };

    fn encrypt_data<P, R, B, S>(
        dir: P,
        rng: &mut R,
        data: B,
        password: S,
        name: Option<&str>,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<String, crate::Error>
    where
        P: AsRef<Path>,
        R: Rng + CryptoRng,
        B: AsRef<[u8]>,
        S: AsRef<[u8]>,
    {
        // let start_time = std::time::Instant::now();
        // Generate a random salt.
        let salt = crate::generate_random_bytes(rng, DEFAULT_KEY_SIZE);
        let iv = crate::generate_random_bytes(rng, DEFAULT_IV_SIZE);

        // Derive the key.
        let kdf = KdfFactory::create(&algorithm, &salt)?;
        let key = kdf.derive_key(password.as_ref())?;
        // Encrypt the private key using AES-128-CTR.

        let ciphertext = cipher::Aes128Ctr::encrypt(&key[..16], &iv[..16], data.as_ref())?;

        // Calculate the MAC.
        let mac = mac::Keccak256Mac.compute(&key, &ciphertext);

        // If a file name is not specified for the keystore, simply use the strigified uuid.
        let id = Uuid::new_v4();
        let name = if let Some(name) = name {
            name.to_string()
        } else {
            id.to_string()
        };

        // Construct and serialize the encrypted JSON keystore.
        let keystore = KeystoreJson {
            id,
            version: 3,
            crypto: CryptoJson {
                cipher: String::from(DEFAULT_CIPHER),
                cipherparams: CipherparamsJson { iv: iv.into() },
                ciphertext: ciphertext.into(),
                kdf: algorithm,
                // kdfparams: KdfParams::Scrypt(ScryptParams {
                //     dklen: DEFAULT_KDF_PARAMS_DKLEN,
                //     n: 2u32.pow(DEFAULT_KDF_PARAMS_LOG_N as u32),
                //     p: DEFAULT_KDF_PARAMS_P,
                //     r: DEFAULT_KDF_PARAMS_R,
                //     salt,
                // }),
                kdfparams: kdf.params(),
                mac: mac.into(),
            },
        };

        let contents = wallet_utils::serde_func::serde_to_string(&keystore)?;

        // Create a file in write-only mode, to store the encrypted JSON keystore.
        let file_name = dir.as_ref().join(name);
        let mut file = File::create(file_name).unwrap();
        file.write_all(contents.as_bytes()).unwrap();

        Ok(id.to_string())
    }

    #[allow(dead_code)]
    async fn scrypt_async(
        password: &[u8],
        salt: &[u8],
        params: &scrypt::Params,
        output: &mut [u8],
    ) -> Result<(), crate::Error> {
        let password = password.to_vec();
        let salt = salt.to_vec();
        let params = params.to_owned();

        let mut output_copy = output.to_vec();
        let output_copy = tokio::task::spawn_blocking(move || {
            scrypt(&password, &salt, &params, &mut output_copy)
                .map_err(KeystoreError::ScryptInvalidOuputLen)?;
            Ok::<Vec<u8>, KeystoreError>(output_copy)
        })
        .await
        .map_err(KeystoreError::TokioTaskJoin)??;
        output.copy_from_slice(&output_copy);
        Ok(())
    }

    fn decrypt_data<P, S>(path: P, password: S) -> Result<Vec<u8>, crate::Error>
    where
        P: AsRef<Path>,
        S: AsRef<[u8]>,
    {
        // Read the file contents as string and deserialize it.
        let mut contents = String::new();
        wallet_utils::file_func::read(&mut contents, path)?;
        let keystore: KeystoreJson = wallet_utils::serde_func::serde_from_str(&contents)?;
        // Derive the key.

        let strategy: Box<dyn KeyDerivationFunction> = match &keystore.crypto.kdfparams {
            KdfParams::Pbkdf2(p) => Box::new(Pbkdf2Kdf::new(p.to_owned())),
            KdfParams::Scrypt(p) => Box::new(ScryptKdf::new(p.to_owned())),
            KdfParams::Argon2id(p) => Box::new(Argon2idKdf::new(p.to_owned())),
        };

        let key = strategy.derive_key(password.as_ref())?;

        // Derive the MAC from the derived key and ciphertext.
        let derived_mac = mac::Keccak256Mac.compute(&key, &keystore.crypto.ciphertext);
        if derived_mac.as_slice() != keystore.crypto.mac.as_slice() {
            return Err(KeystoreError::MacMismatch.into());
        }

        // Decrypt the private key bytes using AES-128-CTR
        let mut data = keystore.crypto.ciphertext;

        cipher::Aes128Ctr::decrypt(
            &key[..16],
            &keystore.crypto.cipherparams.iv[..16],
            &mut data,
        )?;

        Ok(data.0)
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_workflow() {
        let dir = tempdir().expect("create temp dir failed");
        let mut rng = rand::thread_rng();

        // 使用固定测试数据代替真实私钥
        let data_to_encrypt = TEST_PLAINTEXT;

        // 测试所有KDF算法
        for algorithm in [
            // KdfAlgorithm::Pbkdf2,
            KdfAlgorithm::Scrypt,
            KdfAlgorithm::Argon2id,
        ] {
            let file_name = format!("test-key-{:?}", algorithm);
            let keystore_path = dir.path().join(&file_name);

            // 加密测试
            encrypt_data(
                dir.path(),
                &mut rng,
                data_to_encrypt,
                TEST_PASSWORD,
                Some(&file_name),
                algorithm,
            )
            .expect("encryption failed");

            // 验证文件存在
            assert!(keystore_path.exists());

            // 解密测试
            let decrypted = decrypt_data(&keystore_path, TEST_PASSWORD).expect("decryption failed");

            // 验证解密结果
            assert_eq!(decrypted.as_slice(), data_to_encrypt);
        }
    }

    #[test]
    fn test_error_cases() {
        // 测试错误密码
        let dir = tempdir().expect("create temp dir failed");
        let mut rng = rand::thread_rng();
        let file_name = "error-test-key";

        encrypt_data(
            dir.path(),
            &mut rng,
            TEST_PLAINTEXT,
            TEST_PASSWORD,
            Some(file_name),
            KdfAlgorithm::Argon2id,
        )
        .unwrap();

        let keystore_path = dir.path().join(file_name);
        match decrypt_data(&keystore_path, "wrong_password") {
            Err(e) => assert!(matches!(
                e,
                crate::Error::Keystore(KeystoreError::MacMismatch)
            )),
            Ok(_) => panic!("Should have failed with MacMismatch"),
        }
    }

    #[test]
    fn test_kdf_vectors() {
        // 使用官方测试向量验证KDF实现
        let test_vectors = vec![(
            KdfParams::Argon2id(
                Argon2idKdf::recommended_params_with_salt(
                    "e4216e946a28895d743c3f86dc37477ffc006e8e363aea0199c3595d4ff79e4f".as_bytes(),
                )
                .params,
            ),
            "known_password",
            hex::decode("8447df3d3bc2ef6d7bfc653a4244b3c8e7b4f2d216176c8e22f435bcd7e80d3d")
                .unwrap(),
        )];

        for (params, password, expected) in test_vectors {
            let strategy: Box<dyn KeyDerivationFunction> = match &params {
                KdfParams::Pbkdf2(p) => Box::new(Pbkdf2Kdf::new(p.clone())),
                KdfParams::Scrypt(p) => Box::new(ScryptKdf::new(p.clone())),
                KdfParams::Argon2id(p) => Box::new(Argon2idKdf::new(p.clone())),
            };

            let output = strategy.derive_key(password.as_ref()).unwrap();
            assert_eq!(output, expected);
        }
    }

    #[test]
    fn test_hex_decoding() {
        // 测试hex解码功能
        let valid_hex = "48656c6c6f"; // "Hello" 的hex编码
        let decoded = Vec::from_hex(valid_hex).unwrap();
        assert_eq!(decoded, b"Hello");

        let invalid_hex = "ghijkl";
        assert!(Vec::from_hex(invalid_hex).is_err());
    }
}
