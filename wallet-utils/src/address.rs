use crate::error::parse::ParseError;
use alloy::primitives;
use sha2::Digest as _;
use std::str::FromStr;

pub fn bs58_addr_to_hex(bs58_addr: &str) -> Result<String, crate::Error> {
    let bs58_addr = bs58_addr.trim();
    let bytes = bs58::decode(bs58_addr).into_vec().map_err(|_| {
        crate::Error::Parse(ParseError::AddressConvertFailed(bs58_addr.to_string()))
    })?;
    if bytes.len() != 25 {
        return Err(crate::Error::Parse(ParseError::AddressConvertFailed(
            bs58_addr.to_string(),
        )));
    }
    Ok(hex::encode(&bytes[..21]))
}

pub fn bs58_addr_to_hex_bytes(bs58_addr: &str) -> Result<Vec<u8>, crate::Error> {
    bs58::decode(bs58_addr)
        .into_vec()
        .map_err(|_| crate::Error::Parse(ParseError::AddressConvertFailed(bs58_addr.to_string())))
}

pub fn is_tron_address(address: &str) -> bool {
    let address = address.trim();
    if address.len() != 34 || !address.starts_with('T') {
        return false;
    }

    if let Ok(decoded) = bs58::decode(address).into_vec() {
        if decoded.len() == 25 {
            let (data, checksum) = decoded.split_at(21);
            let hash = sha2::Sha256::digest(sha2::Sha256::digest(data));
            return &hash[..4] == checksum;
        }
    }
    false
}

pub fn parse_eth_address(address: &str) -> Result<primitives::Address, crate::Error> {
    primitives::Address::from_str(address.trim()).map_err(|e| {
        crate::Error::Parse(ParseError::AddressConvertFailed(format!(
            "to_eth_address err:{}:address = {}",
            e, address
        )))
    })
}

pub fn parse_sol_address(pubkey: &str) -> Result<solana_sdk::pubkey::Pubkey, crate::Error> {
    solana_sdk::pubkey::Pubkey::from_str(pubkey.trim()).map_err(|e| {
        crate::Error::Parse(ParseError::AddressConvertFailed(format!(
            "to_sol_address err:{}:address = {}",
            e, pubkey
        )))
    })
}

// pub const BIP32_HARDEN: u32 = 2147483648 (0x80000000)
// pub const MAX: Self = 2147483647 (0x7FFFFFFF)
pub fn i32_index_to_hardened_u32(index: i32) -> Result<u32, crate::Error> {
    let index = if index < 0 {
        let positive_index = index
            .checked_add_unsigned(i32::MAX as u32 + 1)
            .ok_or(crate::Error::AddressIndexOverflowOccured)? as u32;
        positive_index | 0x80000000
        // index
        //     .checked_add(i32::MAX + 1)
        //     .ok_or(crate::Error::AddressIndexOverflowOccured)? as u32
    } else {
        index as u32
    };
    Ok(index)
}

pub fn i32_index_to_unhardened_u32(index: i32) -> Result<u32, crate::Error> {
    let index = if index < 0 {
        index
            .checked_add_unsigned(i32::MAX as u32 + 1)
            .ok_or(crate::Error::AddressIndexOverflowOccured)? as u32
    } else {
        index as u32
    };
    Ok(index)
}

pub fn u32_hardened_index_to_i32(hardend_index: u32) -> Result<i32, crate::Error> {
    tracing::debug!("index = {}", hardend_index);

    // 如果是硬化索引
    if hardend_index & 0x80000000 != 0 {
        let unmarked_index = hardend_index & 0x7FFFFFFF; // 去掉硬化标记
        tracing::debug!("unmarked_index = {}", unmarked_index);

        // 计算负数索引值
        let negative_index = if unmarked_index >= (i32::MAX as u32 + 1) {
            unmarked_index
                .checked_sub(i32::MAX as u32 + 1) // 0x80000000 = i32::MAX + 1
                .ok_or(crate::Error::AddressIndexOverflowOccured)? as i32
        } else {
            (unmarked_index as i32)
                .checked_sub_unsigned(i32::MAX as u32 + 1)
                .ok_or(crate::Error::AddressIndexOverflowOccured)?
        };
        // let negative_index = unmarked_index
        //     .checked_sub(0x80000000) // 0x80000000 = i32::MAX + 1
        //     .ok_or(crate::Error::AddressIndexOverflowOccured)? as i32;
        tracing::debug!("negative_index = {}", negative_index);
        Ok(negative_index)
    } else {
        // 非硬化索引直接转换为正数
        Ok(hardend_index as i32)
    }
}

// pub fn u32_index_to_i32(index: u32) -> Result<i32, crate::Error> {
//     // if index > i32::MAX as u32 {

//     tracing::warn!("index = {}", index);
//     if index & 0x80000000 != 0 {
//         let unmarked_index = index & 0x7FFFFFFF;
//         // let negative_index = index
//         let negative_index = unmarked_index.checked_sub(i32::MAX as u32 + 1);
//         tracing::warn!("unmarked_index = {}", unmarked_index);
//         tracing::warn!("negative_index = {:?}", negative_index);
//         let negative_index = negative_index.ok_or(crate::Error::AddressIndexOverflowOccured)?;
//         Ok(negative_index as i32)
//     } else {
//         Ok(index as i32)
//     }
// }

pub fn account_id_to_index(account_id: u32) -> u32 {
    if account_id == 0 {
        u32::MAX
    } else {
        account_id - 1
    }
}

pub fn index_to_account_id(index: u32) -> u32 {
    if index == u32::MAX {
        0
    } else {
        index + 1
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountIndexMap {
    pub account_id: u32,
    pub unhardend_index: u32,
    pub hardened_index: u32,
    pub input_index: i32,
}

impl AccountIndexMap {
    // pub fn new(account_id: u32, index: u32, input_index: i32) -> Self {
    //     Self {
    //         account_id,
    //         hardened_index: index,
    //         input_index,
    //     }
    // }

    pub fn from_input_index(input_index: i32) -> Result<Self, crate::Error> {
        let hardened_index = i32_index_to_hardened_u32(input_index)?;
        let unhardend_index = i32_index_to_unhardened_u32(input_index)?;
        tracing::debug!("hardened_index = {}", hardened_index);
        let account_id = index_to_account_id(hardened_index);
        tracing::debug!("account_id = {}", account_id);
        Ok(Self {
            account_id,
            unhardend_index,
            hardened_index,
            input_index,
        })
    }

    pub fn from_index(index: u32) -> Result<Self, crate::Error> {
        let account_id = index_to_account_id(index);
        let input_index = u32_hardened_index_to_i32(index)?;
        let unhardend_index = i32_index_to_unhardened_u32(input_index)?;
        Ok(Self {
            account_id,
            unhardend_index,
            hardened_index: index,
            input_index,
        })
    }

    pub fn from_account_id(account_id: u32) -> Result<Self, crate::Error> {
        tracing::debug!("account_id = {}", account_id);
        let hardened_index = account_id_to_index(account_id);
        let input_index = u32_hardened_index_to_i32(hardened_index)?;
        let unhardend_index = i32_index_to_unhardened_u32(input_index)?;
        Ok(Self {
            account_id,
            unhardend_index,
            hardened_index,
            input_index,
        })
    }
}
