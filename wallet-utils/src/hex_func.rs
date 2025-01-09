use base64::Engine;

use crate::error::parse::ParseError;
use std::fmt::Debug;

pub fn hex_to_utf8(hex_str: &str) -> Result<String, crate::Error> {
    String::from_utf8(hex_decode(hex_str)?).map_err(|e| crate::Error::Parse(e.into()))
}

pub fn utf8_to_hex(utf8_str: &str) -> String {
    hex::encode(utf8_str)
}

pub fn hex_decode(hex_str: &str) -> Result<Vec<u8>, crate::Error> {
    hex::decode(hex_str).map_err(|e| {
        crate::Error::Parse(ParseError::FromHex(format!(
            "hex decode error: {e} value = {hex_str}"
        )))
    })
}

pub fn hex_encode<T: AsRef<[u8]>>(data: T) -> String {
    hex::encode(data)
}

pub fn bincode_encode<T: serde::Serialize + Debug>(data: &T) -> Result<String, crate::Error> {
    Ok(hex::encode(bincode::serialize(data).map_err(|e| {
        crate::Error::Parse(ParseError::CustomEnum(format!(
            "bincode encode error: {e} value = {data:?}"
        )))
    })?))
}

pub fn bincode_decode<T: serde::de::DeserializeOwned>(data: &str) -> Result<T, crate::Error> {
    bincode::deserialize(&hex_decode(data)?).map_err(|e| {
        crate::Error::Parse(ParseError::CustomEnum(format!(
            "bincode decode error: {e} value = {data}"
        )))
    })
}

pub fn bin_decode_bytes<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, crate::Error> {
    bincode::deserialize(bytes).map_err(|e| {
        crate::Error::Parse(ParseError::CustomEnum(format!("bincode decode error: {e}")))
    })
}

pub fn bin_encode_bytes<T: serde::Serialize + Debug>(data: &T) -> Result<Vec<u8>, crate::Error> {
    bincode::serialize(data).map_err(|e| {
        crate::Error::Parse(ParseError::CustomEnum(format!(
            "bincode encode error: {e} value = {data:?}"
        )))
    })
}

pub fn bs64_encode<T: serde::Serialize + Debug>(data: &T) -> Result<String, crate::Error> {
    Ok(base64::engine::general_purpose::STANDARD.encode(bin_encode_bytes(data)?))
}

pub fn bs64_decode<T: serde::de::DeserializeOwned>(data: &str) -> Result<T, crate::Error> {
    bin_decode_bytes(
        &base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|e| crate::Error::Crypto(e.into()))?,
    )
}

#[cfg(test)]
mod tests {
    use crate::hex_func::{hex_to_utf8, utf8_to_hex};

    #[test]
    fn test_hex_to_utf8() {
        let hex_str = "4163636f756e74207265736f7572636520696e73756666696369656e74206572726f722e";
        let utf8_str = hex_to_utf8(hex_str).unwrap();
        assert_eq!(utf8_str, "我是一个备注信息");
    }

    #[test]
    fn test_utf8_to_hex() {
        let utf8_str = "我是一个备注信息";
        let hex_str = utf8_to_hex(utf8_str);
        assert_eq!(hex_str, "e68891e698afe4b880e4b8aae5a487e6b3a8e4bfa1e681af");
    }
}
