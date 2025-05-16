use crate::error::parse::ParseError;
use alloy::primitives;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
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

pub fn hex_to_bs58_addr(hex_addr: &str) -> Result<String, crate::Error> {
    let bytes = hex::decode(hex_addr)
        .map_err(|_| crate::Error::Parse(ParseError::AddressConvertFailed(hex_addr.to_string())))?;
    if bytes.len() != 21 {
        return Err(crate::Error::Parse(ParseError::AddressConvertFailed(
            hex_addr.to_string(),
        )));
    }
    // 计算校验和 (checksum)
    let hash1 = Sha256::digest(&bytes);
    let hash2 = Sha256::digest(&hash1);
    let checksum = &hash2[..4]; // 校验和为前 4 字节

    let mut full_bytes = bytes;
    full_bytes.extend_from_slice(checksum); // 添加校验和

    // 转换为 Base58 编码
    Ok(bs58::encode(full_bytes).into_string())
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

pub fn eth_addr_to_tron_addr(eth_addr: &str) -> Result<String, crate::Error> {
    let hex_trimmed = eth_addr.trim_start_matches("0x");
    let eth_bytes = hex::decode(hex_trimmed).expect("invalid hex address");

    assert_eq!(eth_bytes.len(), 20);

    // 拼接 0x41 前缀
    let mut tron_raw = vec![0x41];
    tron_raw.extend(eth_bytes);

    // 计算 checksum
    let hash1 = Sha256::digest(&tron_raw);
    let hash2 = Sha256::digest(&hash1);
    let checksum = &hash2[0..4];

    // 拼接后编码
    let mut full = tron_raw.clone();
    full.extend_from_slice(checksum);

    Ok(bs58::encode(full).into_string())
}

pub fn tron_addr_to_eth_addr(tron_addr: &str) -> Result<String, String> {
    let decoded = bs58::decode(tron_addr)
        .into_vec()
        .map_err(|e| format!("Base58 decode error: {:?}", e))?;

    if decoded[0] != 0x41 {
        return Err("Invalid TRON address format".into());
    }

    // 截取后 20 字节
    let eth_bytes = &decoded[1..21];

    Ok(format!("0x{}", hex::encode(eth_bytes)))
}

pub fn parse_sui_address(
    address: &str,
) -> Result<sui_sdk::types::base_types::SuiAddress, crate::Error> {
    sui_sdk::types::base_types::SuiAddress::from_str(address.trim()).map_err(|e| {
        crate::Error::Parse(ParseError::AddressConvertFailed(format!(
            "to_sui_address err:{}:address = {}",
            e, address
        )))
    })
}

pub fn parse_sui_type_tag(s: &str) -> Result<sui_sdk::types::TypeTag, crate::Error> {
    sui_sdk::types::parse_sui_type_tag(s).map_err(|e| {
        crate::Error::Parse(ParseError::AddressConvertFailed(format!(
            "to_sui_address err:{}:address = {}",
            e, s
        )))
    })
}

pub fn parse_sui_struct_tag(
    s: &str,
) -> Result<move_core_types::language_storage::StructTag, crate::Error> {
    sui_sdk::types::parse_sui_struct_tag(s).map_err(|e| {
        crate::Error::Parse(ParseError::AddressConvertFailed(format!(
            "to_sui_address err:{}:address = {}",
            e, s
        )))
    })
}

pub fn parse_object_id_from_hex(
    hex_string: &str,
) -> Result<sui_sdk::types::base_types::ObjectID, crate::Error> {
    sui_sdk::types::base_types::ObjectID::from_hex_literal(hex_string).map_err(|e| {
        crate::Error::Parse(ParseError::AddressConvertFailed(format!(
            "to_sui_address err:{}:address = {}",
            e, hex_string
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

        tracing::debug!("negative_index = {}", negative_index);
        Ok(negative_index)
    } else {
        // 非硬化索引直接转换为正数
        Ok(hardend_index as i32)
    }
}

pub fn account_id_to_index(account_id: u32) -> u32 {
    if account_id == 0 {
        u32::MAX
    } else {
        account_id - 1
    }
}

pub fn index_to_account_id(index: u32) -> u32 {
    if index == u32::MAX { 0 } else { index + 1 }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountIndexMap {
    pub account_id: u32,
    pub unhardend_index: u32,
    pub hardened_index: u32,
    pub input_index: i32,
}

impl AccountIndexMap {
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

pub fn to_checksum_address(address: &str) -> String {
    use sha3::Digest as _;
    // 去掉 0x 前缀并转换为小写
    let address = address.trim_start_matches("0x").to_lowercase();

    // 计算 Keccak-256 哈希
    let hash = sha3::Keccak256::digest(address.as_bytes());

    // 根据哈希值调整字符大小写
    let checksum_address: String = address
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_digit(10) {
                c // 数字保持不变
            } else {
                // 判断对应哈希位的值
                let hash_char = (hash[i / 2] >> (4 * (1 - i % 2))) & 0xF;
                if hash_char >= 8 {
                    c.to_ascii_uppercase() // 大写
                } else {
                    c.to_ascii_lowercase() // 小写
                }
            }
        })
        .collect();

    format!("0x{}", checksum_address)
}

#[cfg(test)]
mod tests {
    use crate::address::{
        AccountIndexMap, hex_to_bs58_addr, parse_sui_address, parse_sui_type_tag,
    };

    use super::{parse_sui_struct_tag, to_checksum_address};

    #[test]
    fn test_from_input_index() {
        let input_index = -3;
        let account_index_map = AccountIndexMap::from_input_index(input_index).unwrap();
        println!("account_index_map: {account_index_map:?}");
        assert_eq!(account_index_map.account_id, 0);
        assert_eq!(account_index_map.hardened_index, 0);
        assert_eq!(account_index_map.unhardend_index, 0);
    }

    #[test]
    fn test_to_checksum_address() {
        let input = "0x3bac24b73c7a03c8715697ca1646a6f85b91023a";
        let expected = "0x3bAc24b73c7A03C8715697cA1646a6f85B91023a";
        assert_eq!(to_checksum_address(input), expected);

        let input = "0xf7d5c082ce49922913404b56168eba82dda4c1f7";
        let expected = "0xF7d5c082Ce49922913404b56168EBa82Dda4c1F7";
        assert_eq!(to_checksum_address(input), expected);

        let input = "0xf1299eb148b413be971822dff4fd079dab9d045d";
        let expected = "0xf1299EB148b413bE971822DfF4fD079dAB9d045d";
        assert_eq!(to_checksum_address(input), expected);
    }

    #[test]
    fn test_hex_to_bs58_addr() {
        // 示例 Hex 地址（21 字节有效数据）
        let hex_addr = "4178c842ee63b253f8f0d2955bbc582c661a078c9d";

        // 预期的 Base58 地址
        let expected_bs58_addr = "TLyqzVGLV1srkB7dToTAEqgDSfPtXRJZYH";

        // 调用函数
        match hex_to_bs58_addr(hex_addr) {
            Ok(bs58_addr) => {
                // 断言结果是否符合预期
                assert_eq!(bs58_addr, expected_bs58_addr, "Base58 地址不正确");
            }
            Err(e) => {
                // 如果函数出错，测试失败
                panic!("函数调用失败: {:?}", e);
            }
        }
    }

    #[test]
    fn test_parse_sui_address() {
        let input =
            "0x1b9e65276fbeab5569a0afb074bb090b9eb867082417b0470a1a04f4be6d2f3a::qtoken::QTOKEN";
        // let expected = "0x3bAc24b73c7A03C8715697cA1646a6f85B91023a";

        let result = parse_sui_type_tag(input);
        println!("result = {result:#?}");
        let tag = result.unwrap();
        let res = tag.to_canonical_string(true);
        println!("res = {res}");
        let res = tag.to_canonical_string(false);
        println!("res = {res}");
        // assert_eq!(to_checksum_address(input), expected);
    }

    #[test]
    fn test_parse_sui_struct_tag() {
        let input =
            "0x1b9e65276fbeab5569a0afb074bb090b9eb867082417b0470a1a04f4be6d2f3a::qtoken::QTOKEN";
        // let expected = "0x3bAc24b73c7A03C8715697cA1646a6f85B91023a";

        let result = parse_sui_struct_tag(input);
        println!("result = {result:#?}");
        let tag = result.unwrap();
    }
}
