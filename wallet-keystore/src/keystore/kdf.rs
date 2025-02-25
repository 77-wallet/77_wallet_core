
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KdfAlgorithm {
    Pbkdf2,
    Scrypt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pbkdf2Params {
    pub c: u32,
    pub dklen: u8,
    pub prf: String,
    // #[serde(with = "crate::utils::hex_bytes")]
    pub salt: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScryptParams {
    pub dklen: u8,
    pub n: u32,
    pub p: u32,
    pub r: u32,
    // #[serde(with = "crate::serde_utils::hex_bytes")]
    pub salt: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KdfParams {
    Pbkdf2(Pbkdf2Params),
    Scrypt(ScryptParams),
}

impl KdfParams {
    pub fn algorithm(&self) -> KdfAlgorithm {
        match self {
            Self::Pbkdf2(_) => KdfAlgorithm::Pbkdf2,
            Self::Scrypt(_) => KdfAlgorithm::Scrypt,
        }
    }
}