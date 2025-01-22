use crate::constant::btc_address_type::*;

use super::category::{BtcAddressCategory, DogAddressCategory, LtcAddressCategory};

use once_cell::sync::Lazy;

pub static BTC_ADDRESS_TYPES: Lazy<Vec<AddressType>> = Lazy::new(|| {
    vec![
        AddressType::Btc(BtcAddressType::P2wpkh),
        AddressType::Btc(BtcAddressType::P2shWpkh),
        AddressType::Btc(BtcAddressType::P2tr),
        AddressType::Btc(BtcAddressType::P2pkh),
    ]
});

pub static LTC_ADDRESS_TYPES: Lazy<Vec<AddressType>> = Lazy::new(|| {
    vec![
        AddressType::Ltc(LtcAddressType::P2wpkh),
        AddressType::Ltc(LtcAddressType::P2shWpkh),
        AddressType::Ltc(LtcAddressType::P2tr),
        AddressType::Ltc(LtcAddressType::P2pkh),
    ]
});

pub static DOG_ADDRESS_TYPES: Lazy<Vec<AddressType>> = Lazy::new(|| {
    vec![
        AddressType::Dog(DogAddressType::P2wpkh),
        AddressType::Dog(DogAddressType::P2shWpkh),
        AddressType::Dog(DogAddressType::P2tr),
        AddressType::Dog(DogAddressType::P2pkh),
    ]
});

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Copy)]
#[serde(untagged)]
pub enum AddressType {
    Btc(BtcAddressType),
    Ltc(LtcAddressType),
    Dog(DogAddressType),
    Other,
}

// impl AddressType {
//     pub fn get_btc_address_types() -> Vec<AddressType> {
//         BTC_ADDRESS_TYPES.to_vec()
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Copy)]
pub enum BtcAddressType {
    /// Pay to public hash (legacy)
    P2pkh,
    /// Pay to script hash
    P2sh,
    /// bech32（Pay to public hash）
    P2shWpkh,
    /// 隔离见证（兼容）
    P2shWsh,
    /// 隔离见证（原生）
    P2wpkh,
    P2wsh,
    /// taproot 单签
    P2tr,
    /// taproot 多签
    P2trSh,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Copy)]
pub enum LtcAddressType {
    /// Pay to public hash (legacy)
    P2pkh,
    /// Pay to script hash
    P2sh,
    /// bech32（Pay to public hash）
    P2shWpkh,
    /// 隔离见证（兼容）
    P2shWsh,
    /// 隔离见证（原生）
    P2wpkh,
    P2wsh,
    /// taproot 单签
    P2tr,
    /// taproot 多签
    P2trSh,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Copy)]
pub enum DogAddressType {
    /// Pay to public hash (legacy)
    P2pkh,
    /// Pay to script hash
    P2sh,
    /// bech32（Pay to public hash）
    P2shWpkh,
    /// 隔离见证（兼容）
    P2shWsh,
    /// 隔离见证（原生）
    P2wpkh,
    P2wsh,
    /// taproot 单签
    P2tr,
    /// taproot 多签
    P2trSh,
}

impl AsRef<str> for BtcAddressType {
    fn as_ref(&self) -> &str {
        match self {
            BtcAddressType::P2pkh => P2PKH,
            BtcAddressType::P2sh => P2SH,
            BtcAddressType::P2shWpkh => P2SH_WPKH,
            BtcAddressType::P2shWsh => P2SH_WSH,
            BtcAddressType::P2wpkh => P2WPKH,
            BtcAddressType::P2wsh => P2WSH,
            BtcAddressType::P2tr => P2TR,
            BtcAddressType::P2trSh => P2TR_SH,
        }
    }
}

impl AsRef<str> for LtcAddressType {
    fn as_ref(&self) -> &str {
        match self {
            LtcAddressType::P2pkh => P2PKH,
            LtcAddressType::P2sh => P2SH,
            LtcAddressType::P2shWpkh => P2SH_WPKH,
            LtcAddressType::P2shWsh => P2SH_WSH,
            LtcAddressType::P2wpkh => P2WPKH,
            LtcAddressType::P2wsh => P2WSH,
            LtcAddressType::P2tr => P2TR,
            LtcAddressType::P2trSh => P2TR_SH,
        }
    }
}

impl AsRef<str> for DogAddressType {
    fn as_ref(&self) -> &str {
        match self {
            DogAddressType::P2pkh => P2PKH,
            DogAddressType::P2sh => P2SH,
            DogAddressType::P2shWpkh => P2SH_WPKH,
            DogAddressType::P2shWsh => P2SH_WSH,
            DogAddressType::P2wpkh => P2WPKH,
            DogAddressType::P2wsh => P2WSH,
            DogAddressType::P2tr => P2TR,
            DogAddressType::P2trSh => P2TR_SH,
        }
    }
}

impl From<BtcAddressCategory> for BtcAddressType {
    fn from(addr_scheme: BtcAddressCategory) -> Self {
        match addr_scheme {
            BtcAddressCategory::Legacy => BtcAddressType::P2sh,
            BtcAddressCategory::NestedSegWit => BtcAddressType::P2shWsh,
            BtcAddressCategory::NativeSegWit => BtcAddressType::P2wsh,
            BtcAddressCategory::Taproot => BtcAddressType::P2trSh,
        }
    }
}

impl From<LtcAddressCategory> for LtcAddressType {
    fn from(addr_scheme: LtcAddressCategory) -> Self {
        match addr_scheme {
            LtcAddressCategory::Legacy => LtcAddressType::P2sh,
            LtcAddressCategory::NestedSegWit => LtcAddressType::P2shWsh,
            LtcAddressCategory::NativeSegWit => LtcAddressType::P2wsh,
            LtcAddressCategory::Taproot => LtcAddressType::P2trSh,
        }
    }
}

impl From<DogAddressCategory> for DogAddressType {
    fn from(addr_scheme: DogAddressCategory) -> Self {
        match addr_scheme {
            DogAddressCategory::Legacy => DogAddressType::P2sh,
            DogAddressCategory::NestedSegWit => DogAddressType::P2shWsh,
            DogAddressCategory::NativeSegWit => DogAddressType::P2wsh,
            DogAddressCategory::Taproot => DogAddressType::P2trSh,
        }
    }
}

impl std::fmt::Display for DogAddressType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl std::fmt::Display for LtcAddressType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl std::fmt::Display for BtcAddressType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl std::fmt::Display for AddressType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressType::Btc(btc_address_type) => write!(f, "{}", btc_address_type),
            AddressType::Ltc(ltc_address_type) => write!(f, "{}", ltc_address_type),
            AddressType::Dog(dog_address_type) => write!(f, "{}", dog_address_type),
            AddressType::Other => write!(f, ""),
        }
    }
}

impl TryFrom<Option<String>> for AddressType {
    type Error = crate::Error;
    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        match value {
            Some(v) => BtcAddressType::try_from(v.as_str()).map(AddressType::Btc),
            None => Ok(AddressType::Other),
        }
    }
}

impl AsRef<str> for AddressType {
    fn as_ref(&self) -> &str {
        match self {
            AddressType::Btc(btc_address_type) => btc_address_type.as_ref(),
            AddressType::Ltc(ltc_address_type) => ltc_address_type.as_ref(),
            AddressType::Dog(dog_address_type) => dog_address_type.as_ref(),
            AddressType::Other => "",
        }
    }
}

impl<T: AsRef<str>> TryFrom<Option<T>> for BtcAddressType {
    type Error = crate::Error;
    fn try_from(value: Option<T>) -> Result<Self, Self::Error> {
        match value {
            Some(v) => BtcAddressType::try_from(v.as_ref()),
            None => Err(crate::Error::BtcNeedAddressType),
        }
    }
}

impl TryFrom<&str> for BtcAddressType {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value.to_lowercase().as_ref() {
            P2PKH => BtcAddressType::P2pkh,
            P2SH => BtcAddressType::P2sh,
            P2SH_WPKH => BtcAddressType::P2shWpkh,
            P2SH_WSH => BtcAddressType::P2shWsh,
            P2WPKH => BtcAddressType::P2wpkh,
            P2WSH => BtcAddressType::P2wsh,
            P2TR => BtcAddressType::P2tr,
            P2TR_SH => BtcAddressType::P2trSh,
            other => return Err(crate::Error::BtcAddressTypeInvalid(other.to_string())),
        })
    }
}

impl<T: AsRef<str>> TryFrom<Option<T>> for LtcAddressType {
    type Error = crate::Error;
    fn try_from(value: Option<T>) -> Result<Self, Self::Error> {
        match value {
            Some(v) => LtcAddressType::try_from(v.as_ref()),
            None => Err(crate::Error::BtcNeedAddressType),
        }
    }
}

impl TryFrom<&str> for LtcAddressType {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value.to_lowercase().as_ref() {
            P2PKH => LtcAddressType::P2pkh,
            P2SH => LtcAddressType::P2sh,
            P2SH_WPKH => LtcAddressType::P2shWpkh,
            P2SH_WSH => LtcAddressType::P2shWsh,
            P2WPKH => LtcAddressType::P2wpkh,
            P2WSH => LtcAddressType::P2wsh,
            P2TR => LtcAddressType::P2tr,
            P2TR_SH => LtcAddressType::P2trSh,
            other => return Err(crate::Error::LtcAddressTypeInvalid(other.to_string())),
        })
    }
}

impl<T: AsRef<str>> TryFrom<Option<T>> for DogAddressType {
    type Error = crate::Error;
    fn try_from(value: Option<T>) -> Result<Self, Self::Error> {
        match value {
            Some(v) => DogAddressType::try_from(v.as_ref()),
            None => Err(crate::Error::BtcNeedAddressType),
        }
    }
}

impl TryFrom<&str> for DogAddressType {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value.to_lowercase().as_ref() {
            P2PKH => DogAddressType::P2pkh,
            P2SH => DogAddressType::P2sh,
            P2SH_WPKH => DogAddressType::P2shWpkh,
            P2SH_WSH => DogAddressType::P2shWsh,
            P2WPKH => DogAddressType::P2wpkh,
            P2WSH => DogAddressType::P2wsh,
            P2TR => DogAddressType::P2tr,
            P2TR_SH => DogAddressType::P2trSh,
            other => return Err(crate::Error::DogAddressTypeInvalid(other.to_string())),
        })
    }
}

// impl AddressType {
//     pub fn get_btc_address_types() -> Vec<AddressType> {
//         BTC_ADDRESS_TYPES.to_vec()
//     }
// }
