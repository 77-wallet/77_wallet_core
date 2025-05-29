use super::r#type::{AddressType, BtcAddressType, DogAddressType, LtcAddressType, TonAddressType};
use crate::constant::btc_address_catecory::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Copy)]
#[serde(untagged)]
pub enum AddressCategory {
    Btc(BtcAddressCategory),
    Ltc(LtcAddressCategory),
    Dog(DogAddressCategory),
    Ton(TonAddressType),
    Other,
}
impl AddressCategory {
    // 展示分类下的名称
    pub fn show_name(&self) -> &str {
        match self {
            AddressCategory::Btc(addr_type) => addr_type.as_ref(),
            AddressCategory::Ltc(addr_type) => addr_type.as_ref(),
            AddressCategory::Dog(addr_type) => addr_type.as_ref(),
            AddressCategory::Ton(addr_type) => addr_type.as_ref(),
            AddressCategory::Other => "",
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Copy)]
pub enum LtcAddressCategory {
    Taproot,
    #[serde(rename = "Nested SegWit")]
    NestedSegWit,
    #[serde(rename = "Native SegWit")]
    NativeSegWit,
    Legacy,
}

impl AsRef<str> for LtcAddressCategory {
    fn as_ref(&self) -> &str {
        match self {
            LtcAddressCategory::Taproot => TAPROOT,
            LtcAddressCategory::NestedSegWit => NESTED_SEG_WIT,
            LtcAddressCategory::NativeSegWit => NATIVE_SEG_WIT,
            LtcAddressCategory::Legacy => LEGACY,
        }
    }
}
impl TryFrom<String> for LtcAddressCategory {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            TAPROOT => Ok(LtcAddressCategory::Taproot),
            NESTED_SEG_WIT => Ok(LtcAddressCategory::NestedSegWit),
            NATIVE_SEG_WIT => Ok(LtcAddressCategory::NativeSegWit),
            LEGACY => Ok(LtcAddressCategory::Legacy),
            other => Err(crate::Error::LtcAddressCategoryInvalid(other.to_string())),
        }
    }
}

impl std::fmt::Display for LtcAddressCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LtcAddressCategory::Taproot => TAPROOT,
            LtcAddressCategory::NestedSegWit => NESTED_SEG_WIT,
            LtcAddressCategory::NativeSegWit => NATIVE_SEG_WIT,
            LtcAddressCategory::Legacy => LEGACY,
        })
    }
}

impl From<LtcAddressType> for LtcAddressCategory {
    fn from(addr_type: LtcAddressType) -> Self {
        match addr_type {
            LtcAddressType::P2pkh | LtcAddressType::P2sh => LtcAddressCategory::Legacy,
            LtcAddressType::P2shWpkh | LtcAddressType::P2shWsh => LtcAddressCategory::NestedSegWit,
            LtcAddressType::P2wpkh | LtcAddressType::P2wsh => LtcAddressCategory::NativeSegWit,
            LtcAddressType::P2tr | LtcAddressType::P2trSh => LtcAddressCategory::Taproot,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Copy)]
pub enum DogAddressCategory {
    Taproot,
    #[serde(rename = "Nested SegWit")]
    NestedSegWit,
    #[serde(rename = "Native SegWit")]
    NativeSegWit,
    Legacy,
}

impl AsRef<str> for DogAddressCategory {
    fn as_ref(&self) -> &str {
        match self {
            DogAddressCategory::Taproot => TAPROOT,
            DogAddressCategory::NestedSegWit => NESTED_SEG_WIT,
            DogAddressCategory::NativeSegWit => NATIVE_SEG_WIT,
            DogAddressCategory::Legacy => LEGACY,
        }
    }
}
impl TryFrom<String> for DogAddressCategory {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            TAPROOT => Ok(DogAddressCategory::Taproot),
            NESTED_SEG_WIT => Ok(DogAddressCategory::NestedSegWit),
            NATIVE_SEG_WIT => Ok(DogAddressCategory::NativeSegWit),
            LEGACY => Ok(DogAddressCategory::Legacy),
            other => Err(crate::Error::DogAddressCategoryInvalid(other.to_string())),
        }
    }
}

impl std::fmt::Display for DogAddressCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DogAddressCategory::Taproot => TAPROOT,
            DogAddressCategory::NestedSegWit => NESTED_SEG_WIT,
            DogAddressCategory::NativeSegWit => NATIVE_SEG_WIT,
            DogAddressCategory::Legacy => LEGACY,
        })
    }
}

impl From<DogAddressType> for DogAddressCategory {
    fn from(addr_type: DogAddressType) -> Self {
        match addr_type {
            DogAddressType::P2pkh | DogAddressType::P2sh => DogAddressCategory::Legacy,
            DogAddressType::P2shWpkh | DogAddressType::P2shWsh => DogAddressCategory::NestedSegWit,
            DogAddressType::P2wpkh | DogAddressType::P2wsh => DogAddressCategory::NativeSegWit,
            DogAddressType::P2tr | DogAddressType::P2trSh => DogAddressCategory::Taproot,
        }
    }
}

impl From<AddressType> for AddressCategory {
    fn from(address: AddressType) -> Self {
        match address {
            AddressType::Btc(addr_type) => AddressCategory::Btc(addr_type.into()),
            AddressType::Ltc(addr_type) => AddressCategory::Ltc(addr_type.into()),
            AddressType::Dog(addr_type) => AddressCategory::Dog(addr_type.into()),
            AddressType::Ton(addr_type) => AddressCategory::Ton(addr_type),
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

    #[test]
    fn test_show_name() {
        let btc_category = AddressCategory::Btc(BtcAddressCategory::Taproot);

        println!("{}", btc_category.show_name());
    }
}
