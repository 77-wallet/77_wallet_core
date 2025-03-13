#![cfg_attr(docsrs, feature(doc_cfg))]
//! A minimalist library to interact with encrypted JSON keystores as per the
//! [Web3 Secret Storage Definition](https://github.com/ethereum/wiki/wiki/Web3-Secret-Storage-Definition).

pub mod argon2id;
pub mod pbkdf2;
pub mod scrypt_;

use argon2id::Argon2idParams;
use pbkdf2::Pbkdf2Params;
use scrypt_::ScryptParams;

use crate::{error::crypto::KeystoreError, KdfAlgorithm};

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
    use uuid::Uuid;

    use std::{fs::File, io::Write, path::Path};

    use crate::{error::crypto::KeystoreError, keystore::factory::KdfFactory};

    const DEFAULT_CIPHER: &str = "aes-128-ctr";
    const DEFAULT_KEY_SIZE: usize = 32usize;
    const DEFAULT_IV_SIZE: usize = 16usize;

    use hex::FromHex;

    use crate::KdfAlgorithm;

    use super::{
        argon2id::Argon2idKdf, pbkdf2::Pbkdf2Kdf, scrypt_::ScryptKdf, KdfParams,
        KeyDerivationFunction,
    };

    /// Encrypts the given private key using the [Scrypt](https://tools.ietf.org/html/rfc7914.html)
    /// password-based key derivation function, and stores it in the provided directory. On success, it
    /// returns the `id` (Uuid) generated for this keystore.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use eth_keystore::encrypt_key;
    /// use rand::RngCore;
    /// use std::path::Path;
    ///
    /// # async fn foobar() -> Result<(), Box<dyn std::error::Error>> {
    /// let dir = Path::new("./keys");
    /// let mut rng = rand::thread_rng();
    ///
    /// // Construct a 32-byte random private key.
    /// let mut private_key = vec![0u8; 32];
    /// rng.fill_bytes(private_key.as_mut_slice());
    ///
    /// // Since we specify a custom filename for the keystore, it will be stored in `$dir/my-key`
    /// let name = encrypt_key(&dir, &mut rng, &private_key, "password_to_keystore", Some("my-key"))?;
    /// # Ok(())
    /// # }
    /// ```
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
        // tracing::info!("[encrypt_data] encrypting data");
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
        // tracing::info!(
        //     "[encrypt_data] mac: {} (elapsed: {}ms)",
        //     hex::encode(&mac),
        //     start_time.elapsed().as_millis()
        // );
        // If a file name is not specified for the keystore, simply use the strigified uuid.
        let id = Uuid::new_v4();
        let name = if let Some(name) = name {
            name.to_string()
        } else {
            id.to_string()
        };
        // tracing::info!(
        //     "[encrypt_data] iv: {iv:?} (elapsed: {}ms)",
        //     start_time.elapsed().as_millis()
        // );

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

        // tracing::info!(
        //     "[encrypt_data] keystore: {:?} (elapsed: {}ms)",
        //     keystore,
        //     start_time.elapsed().as_millis()
        // );
        let contents = wallet_utils::serde_func::serde_to_string(&keystore)?;

        // Create a file in write-only mode, to store the encrypted JSON keystore.
        let file_name = dir.as_ref().join(name);
        // std::thread::spawn(move || {

        // tracing::warn!(
        //     "[encrypt_data] file_name: {file_name:?} (elapsed: {}ms)",
        //     start_time.elapsed().as_millis()
        // );
        let mut file = File::create(file_name).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        // });

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
    async fn test_encrypt_key() {
        let dir = Path::new("");
        let mut rng = rand::thread_rng();

        // let provided_master_key_hex =
        //     "8b09ab2bfb613458f9362c4b79ff5ac8b8c6da10f25017807aa08cea969cd1ca";
        let seed_hex = "e61b56077fd615fa661b720d3021627d37bee396dcebd11a31f51355259712fe3b92f4cbd923dca32d6a80dfafbc0dd8f25a59aff331749c9afeef097a29d5d6";
        let provided_master_key_bytes = hex::decode(seed_hex).unwrap();
        let data_to_encrypt = provided_master_key_bytes.as_slice();

        // // let data_to_encrypt = b"Hello, world!";
        // // let data_to_encrypt = alloy::hex!("e61b56077fd615fa661b720d3021627d37bee396dcebd11a31f51355259712fe3b92f4cbd923dca32d6a80dfafbc0dd8f25a59aff331749c9afeef097a29d5d6");
        // let data_to_encrypt =
        //     alloy::hex!("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80");
        // // let data_to_encrypt = alloy::hex!("hellohellohellohellohello");

        let a = data_to_encrypt.to_vec();
        tracing::info!("[test_encrypt_key] a: {a:?}");
        tracing::info!("[test_encrypt_key] data: {data_to_encrypt:?}");

        tracing::info!("");
        let password = "password_to_keystore";

        // Since we specify a custom filename for the keystore, it will be stored in `$dir/my-key`
        let _name = encrypt_data(
            &dir,
            &mut rng,
            &data_to_encrypt,
            password,
            Some("my-key"),
            KdfAlgorithm::Pbkdf2,
        )
        .unwrap();

        let path = "./my-key";
        let data = decrypt_data(path, password).unwrap();

        let data = hex::encode(data);

        // let wallet = Wallet::from_slice(&data).unwrap();
        // let key = wallet.signer().to_bytes();
        // let private_key = key.to_vec();
        // let private_key = alloy::hex::encode(private_key);
        // let data = alloy::hex::encode(&data);
        tracing::info!("data: {data}");
    }

    #[test]
    fn decrypt() {
        let subs_dir =
            "./tron-TFzMRRzQFhY9XFS37veoswLRuWLNtbyhiB-m%2F44%27%2F195%27%2F0%27%2F0%2F0-pk";
        // let res = KeystoreBuilder::new_decrypt(subs_dir, "q1111111").load();
        let res = decrypt_data(subs_dir, "q1111111").unwrap();
        println!("res: {res:?}");
    }

    #[test]
    fn test_hex_decode() {
        let data = "f1446ee3758d62d2b793ce3834950d10";
        let res = Vec::from_hex(data).unwrap();
        println!("res: {res:?}");
    }

    // #[test]
    // fn test_log_n() {
    //     use rust_decimal::prelude::*;
    //     let n = 8192;
    //     let n_decimal = rust_decimal::Decimal::from(n);
    //     let log2_decomal = n_decimal.log10() / Decimal::from(2).log10();

    //     let log_n = log2_decomal.to_u8().unwrap();

    //     println!("log_n: {log_n}");

    //     let mut log_n = 0;
    //     let mut value = n;
    //     while value > 1 {
    //         value >>= 1; // 右移一位，相当于除以2
    //         log_n += 1;
    //     }
    //     println!("log_n: {log_n}");
    // }
}
