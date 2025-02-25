// src/serde_utils/mod.rs
use hex::{FromHex, ToHex};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexBytes(pub Vec<u8>);

impl HexBytes {
    pub fn to_hex(&self) -> String {
        self.0.encode_hex()
    }
}

impl Serialize for HexBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for HexBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Vec::from_hex(&s)
            .map(HexBytes)
            .map_err(|e| Error::custom(e.to_string()))
    }
}