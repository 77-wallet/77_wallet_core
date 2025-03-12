use hex::{FromHex as _, ToHex};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::crypto::kdfs::{
    argon2id::Argon2idKdf, pbkdf2::Pbkdf2Kdf, scrypt_::ScryptKdf, KeyDerivation,
};

use super::json::KeystoreJson;

const DEFAULT_KDF_PARAMS_DKLEN: u8 = 32u8;
const DEFAULT_KDF_PARAMS_LOG_N: u8 = 10u8;
const DEFAULT_KDF_PARAMS_R: u32 = 8u32;
const DEFAULT_KDF_PARAMS_P: u32 = 1u32;

const TIME_COST: u32 = 3;
const MEMORY_COST: u32 = 65536;
const PARALLELISM: u32 = 1;

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
    ) -> Result<Box<dyn KeyDerivation>, crate::Error> {
        match algorithm {
            KdfAlgorithm::Scrypt => {
                let params = ScryptParams::new(
                    DEFAULT_KDF_PARAMS_DKLEN,
                    2u32.pow(DEFAULT_KDF_PARAMS_LOG_N as u32),
                    DEFAULT_KDF_PARAMS_R,
                    DEFAULT_KDF_PARAMS_P,
                    salt,
                );

                Ok(Box::new(ScryptKdf::new(params)))
            }
            KdfAlgorithm::Pbkdf2 => {
                todo!()
            }
            KdfAlgorithm::Argon2id => {
                let params = Argon2idParams::new(
                    DEFAULT_KDF_PARAMS_DKLEN,
                    TIME_COST,
                    MEMORY_COST,
                    PARALLELISM,
                    salt,
                );

                Ok(Box::new(Argon2idKdf::new(params)))
            }
        }
    }

    pub fn create_from_file(
        keystore: &KeystoreJson,
    ) -> Result<Box<dyn KeyDerivation>, crate::Error> {
        let kdf: Box<dyn KeyDerivation> = match &keystore.crypto.kdfparams {
            KdfParams::Pbkdf2(p) => Box::new(Pbkdf2Kdf::new(p.to_owned())),
            KdfParams::Scrypt(p) => Box::new(ScryptKdf::new(p.to_owned())),
            KdfParams::Argon2id(p) => Box::new(Argon2idKdf::new(p.to_owned())),
        };

        Ok(kdf)
    }

    fn default_scrypt_params() -> KdfParams {
        KdfParams::Scrypt(ScryptParams {
            dklen: 32,
            n: 16384,
            r: 8,
            p: 1,
            salt: vec![],
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pbkdf2Params {
    pub c: u32,
    pub dklen: u8,
    pub prf: String,
    #[serde(serialize_with = "buffer_to_hex", deserialize_with = "hex_to_buffer")]
    pub salt: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScryptParams {
    pub dklen: u8,
    pub n: u32,
    pub p: u32,
    pub r: u32,
    #[serde(serialize_with = "buffer_to_hex", deserialize_with = "hex_to_buffer")]
    pub salt: Vec<u8>,
}

impl ScryptParams {
    pub(crate) fn new(dklen: u8, n: u32, r: u32, p: u32, salt: &[u8]) -> Self {
        Self {
            dklen,
            n,
            r,
            p,
            salt: salt.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Argon2idParams {
    pub dklen: u8,        // 输出密钥长度
    pub time_cost: u32,   // 迭代次数
    pub memory_cost: u32, // 内存成本（单位：KB）
    pub parallelism: u32, // 并行度
    pub salt: Vec<u8>,    // 盐值
}

impl Argon2idParams {
    pub fn new(dklen: u8, time_cost: u32, memory_cost: u32, parallelism: u32, salt: &[u8]) -> Self {
        Self {
            dklen,
            time_cost,
            memory_cost,
            parallelism,
            salt: salt.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

pub(crate) fn buffer_to_hex<T, S>(buffer: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    serializer.serialize_str(&buffer.encode_hex::<String>())
}

pub(crate) fn hex_to_buffer<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| Vec::from_hex(string).map_err(|err| Error::custom(err.to_string())))
}
