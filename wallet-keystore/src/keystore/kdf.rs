use hex::{FromHex as _, ToHex};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Types of key derivition functions supported by the Web3 Secret Storage.
pub enum KdfAlgorithm {
    Pbkdf2,
    Scrypt,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
