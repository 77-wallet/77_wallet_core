use blake2::{Blake2b, Digest};
use ed25519_dalek::SigningKey;
use std::convert::TryInto;

const ED25519_FLAG: u8 = 0x00; // Ed25519 地址标识

/// SLIP-0010 密钥派生（逐步骤调试）
pub fn slip0010_derive_ed25519(seed: &[u8], path: &str) -> Result<[u8; 32], crate::Error> {
    // 初始化 HMAC
    let mut key = wallet_utils::parse_func::hmac_sha512(b"ed25519 seed", seed)?;

    // 处理路径段
    for segment in path.split('/').skip(1) {
        let index = parse_segment(segment)?;

        // 构造 HMAC 输入数据（严格 SLIP-0010）
        let mut data = Vec::new();
        data.push(0x00); // 硬化标记
        data.extend_from_slice(&key[..32]); // 链码
        data.extend_from_slice(&index.to_be_bytes()); // 大端序索引

        // 计算新密钥
        key = wallet_utils::parse_func::hmac_sha512(&key[32..], &data)?;
    }
    key[..32].try_into().map_err(|e| {
        crate::Error::Utils(wallet_utils::Error::Parse(
            wallet_utils::ParseError::TryFromSliceError(e),
        ))
    })
}

/// 从原始公钥字节生成地址（关键验证函数）
pub(crate) fn generate_sui_address_from_bytes(pub_key: &[u8]) -> String {
    let mut hasher_input = [0u8; 33];
    hasher_input[0] = ED25519_FLAG; // 0x00
    hasher_input[1..].copy_from_slice(pub_key);

    let hash: generic_array::GenericArray<u8, typenum::U32> = Blake2b::digest(&hasher_input);
    format!("0x{}", hex::encode(hash))
}

/// 路径解析（带硬化标记处理）
fn parse_segment(segment: &str) -> Result<u32, crate::Error> {
    let (num_str, hardened) = segment
        .strip_suffix('\'')
        .map_or((segment, false), |s| (s, true));

    num_str
        .parse::<u32>()
        .map(|n| n | (hardened as u32) << 31)
        .map_err(|e| crate::Error::Utils(wallet_utils::Error::Parse(e.into())))
}

pub(crate) fn get_pub_key(key: [u8; 32]) -> Result<ed25519_dalek::VerifyingKey, crate::Error> {
    let private_key = SigningKey::from_bytes(&key[..32].try_into().map_err(
        |e: std::array::TryFromSliceError| {
            crate::Error::Utils(wallet_utils::Error::Parse(e.into()))
        },
    )?);
    Ok(private_key.verifying_key())
}

#[cfg(test)]
mod tests {
    use super::*;
    use coins_bip39::{English, Mnemonic};
    const SUI_COIN_TYPE: u32 = 784 | 0x80000000; // 784' 的硬化编码
    const BIP44_PURPOSE: u32 = 44 | 0x80000000; // 44' 的硬化编码

    /// 主函数（完整路径构造）
    fn get_sui_address(seed: &[u8], path: &str) -> Result<String, crate::Error> {
        println!("[0] Seed: {}", hex::encode(seed));
        // 3. 密钥派生
        let key = slip0010_derive_ed25519(&seed, path)?;

        println!("   |__ Private_key: {}", hex::encode(key));
        let pub_key = get_pub_key(key)?;

        println!("pub_key: {}", hex::encode(pub_key.as_bytes()));
        // 4. 生成地址
        Ok(generate_sui_address_from_bytes(pub_key.as_bytes()))
    }

    #[test]
    fn test_official_vector() {
        let mnemonic = "";
        // "film crazy soon outside stand loop subway crumble thrive popular green nuclear struggle pistol arm wife phrase warfare march wheat nephew ask sunny firm"
        // 1. 生成 BIP-39 种子（空密码）
        let mnemonic =
            Mnemonic::<English>::new_from_phrase(mnemonic).expect("Invalid mnemonic phrase");
        let seed = mnemonic.to_seed(Some("")).unwrap(); // 注意：必须使用 Some("")

        // 2. 构造完整派生路径
        let path = format!(
            "m/{}'/{}'/0'/0'/0'",        // 官方测试用例路径
            BIP44_PURPOSE & !0x80000000, // 显示逻辑值 44'
            SUI_COIN_TYPE & !0x80000000  // 显示逻辑值 784'
        );

        let address = get_sui_address(&seed, &path).unwrap();

        println!("address: {}", address);
        // "suiprivkey1qr4w9sqf2dlq9uwpml6gtyr9mwhwlgyc40nnpf8uk5k9yuzt0q29vep62tu";
        // assert_eq!(
        //     address,
        //     // "0xa2d14fad60c56049ecf75246a481934691214ce413e6a8ae2fe6834c173a6133"
        //     "0x885f29a4f1b4d63822728a1b1811d0278c4e25f27d3754ddd387cd34f9482d0f"
        // );
    }

    #[test]
    fn test_key_encoding() {
        let raw_private =
            hex::decode("eae2c009537e02f1c1dff4859065dbaeefa098abe730a4fcb52c52704b781456")
                .unwrap();
        let full_key = [&[0x00], &raw_private[..]].concat();

        let data = bech32::primitives::hrp::Hrp::parse("suiprivkey").unwrap();
        let data = bech32::encode::<bech32::Bech32>(data, &full_key).unwrap();

        println!("data: {}", data);
        assert_eq!(
            data,
            "suiprivkey1qr4w9sqf2dlq9uwpml6gtyr9mwhwlgyc40nnpf8uk5k9yuzt0q29vep62tu"
        );
    }

    #[test]
    fn test_pub_key() {
        // 输入参数
        let base64_pub_key = "ACJkf+7vNjBgvUIFoWcaFfEKEjZ2WRixtfY42C8zz8Rp";
        let expected_address = "0xa2d14fad60c56049ecf75246a481934691214ce413e6a8ae2fe6834c173a6133";
        // 步骤 1：Base64 解码
        let pub_key_bytes = wallet_utils::base64_to_bytes(base64_pub_key).unwrap();
        println!("[1] Base64 decoded: {}", hex::encode(&pub_key_bytes));
        // 输出: 0022647feeef363060bd4205a1671a15f10a1236765918b1b5f638d82f33cfc469

        // 步骤 2：验证字节结构
        assert_eq!(
            pub_key_bytes.len(),
            33,
            "公钥应为 33 字节（flag + raw key）"
        );
        assert_eq!(pub_key_bytes[0], 0x00, "首字节应为 Ed25519 标志位 0x00");

        // 步骤 3：提取原始公钥（后 32 字节）
        let raw_pub_key = &pub_key_bytes[1..];
        println!("[2] Raw public key:  {}", hex::encode(raw_pub_key));
        // 输出: 22647feeef363060bd4205a1671a15f10a1236765918b1b5f638d82f33cfc469

        // 步骤 4：生成地址
        let address = generate_sui_address_from_bytes(raw_pub_key);
        println!("[3] Generated address: {}", address);
        assert_eq!(address, expected_address);
    }
}
