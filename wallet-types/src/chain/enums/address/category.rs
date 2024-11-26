use crate::constant::btc_address_catecory::*;

use super::r#type::{AddressType, BtcAddressType};
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Copy)]
#[serde(untagged)]
pub enum AddressCategory {
    Btc(BtcAddressCategory),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Copy)]
pub enum BtcAddressCategory {
    Taproot,
    #[serde(rename = "Nested SegWit")]
    NestedSegWit,
    #[serde(rename = "Native SegWit")]
    NativeSegWit,
    Legacy,
}

impl AsRef<str> for BtcAddressCategory {
    fn as_ref(&self) -> &str {
        match self {
            BtcAddressCategory::Taproot => TAPROOT,
            BtcAddressCategory::NestedSegWit => NESTED_SEG_WIT,
            BtcAddressCategory::NativeSegWit => NATIVE_SEG_WIT,
            BtcAddressCategory::Legacy => LEGACY,
        }
    }
}
impl TryFrom<String> for BtcAddressCategory {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            TAPROOT => Ok(BtcAddressCategory::Taproot),
            NESTED_SEG_WIT => Ok(BtcAddressCategory::NestedSegWit),
            NATIVE_SEG_WIT => Ok(BtcAddressCategory::NativeSegWit),
            LEGACY => Ok(BtcAddressCategory::Legacy),
            other => Err(crate::Error::BtcAddressCategoryInvalid(other.to_string())),
        }
    }
}

impl std::fmt::Display for BtcAddressCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BtcAddressCategory::Taproot => TAPROOT,
            BtcAddressCategory::NestedSegWit => NESTED_SEG_WIT,
            BtcAddressCategory::NativeSegWit => NATIVE_SEG_WIT,
            BtcAddressCategory::Legacy => LEGACY,
        })
    }
}

impl From<BtcAddressType> for BtcAddressCategory {
    fn from(addr_type: BtcAddressType) -> Self {
        match addr_type {
            BtcAddressType::P2pkh | BtcAddressType::P2sh => BtcAddressCategory::Legacy,
            BtcAddressType::P2shWpkh | BtcAddressType::P2shWsh => BtcAddressCategory::NestedSegWit,
            BtcAddressType::P2wpkh | BtcAddressType::P2wsh => BtcAddressCategory::NativeSegWit,
            BtcAddressType::P2tr | BtcAddressType::P2trSh => BtcAddressCategory::Taproot,
        }
    }
}

impl From<AddressType> for AddressCategory {
    fn from(address: AddressType) -> Self {
        match address {
            AddressType::Btc(addr_type) => AddressCategory::Btc(addr_type.into()),
            AddressType::Other => AddressCategory::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize_address_category() {
        // 测试 AddressCategory::Btc 变体的序列化
        let btc_category = AddressCategory::Btc(BtcAddressCategory::Taproot);
        let btc_serialized = serde_json::to_string(&btc_category).unwrap();
        assert_eq!(btc_serialized, "\"Taproot\"");

        // 测试 AddressCategory::Other 变体的序列化
        let other_category = AddressCategory::Other;
        let other_serialized = serde_json::to_string(&other_category).unwrap();
        assert_eq!(other_serialized, "null");

        let test: Option<String> = None;
        let test_serialized = serde_json::to_string(&test).unwrap();
        assert_eq!(test_serialized, "null");
    }
}
