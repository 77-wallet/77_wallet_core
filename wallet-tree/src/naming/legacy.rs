use serde::Serialize;

use super::{FileMeta, FileType, NamingStrategy};

#[derive(Debug, Clone)]
pub struct LegacyFileMeta {
    // pub directory_naming: DirectoryNaming,
    pub file_type: FileType,
    pub address: String,
    pub chain_code: Option<String>,
    pub derivation_path: Option<String>,
    // pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl FileMeta for LegacyFileMeta {
    fn file_type(&self) -> &FileType {
        &self.file_type
    }
    fn address(&self) -> Option<String> {
        Some(self.address.clone())
    }

    fn chain_code(&self) -> Option<String> {
        self.chain_code.clone()
    }
    fn derivation_path(&self) -> Option<String> {
        self.derivation_path.clone()
    }

    fn account_index(&self) -> Option<u32> {
        None
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize)]
pub struct LegacyNaming;

impl NamingStrategy for LegacyNaming {
    fn encode(&self, meta: Box<dyn FileMeta>) -> Result<String, crate::Error> {
        match meta.file_type() {
            FileType::Phrase => Ok(format!(
                "{}-phrase",
                meta.address().ok_or(crate::Error::MissingAddress)?
            )),
            FileType::PrivateKey => Ok(format!(
                "{}-pk",
                meta.address().ok_or(crate::Error::MissingAddress)?
            )),
            FileType::Seed => Ok(format!(
                "{}-seed",
                meta.address().ok_or(crate::Error::MissingAddress)?
            )),
            FileType::DerivedData => {
                let chain = meta.chain_code();
                let chain = chain.as_ref().ok_or(crate::Error::MissingChainCode)?;
                let derivation_path = meta.derivation_path();

                let derivation_path = derivation_path
                    .as_ref()
                    .ok_or(crate::Error::MissingDerivation)?;
                let encoded_path =
                    wallet_utils::parse_func::derivation_path_percent_encode(derivation_path);
                Ok(format!(
                    "{}-{}-{}-pk",
                    chain,
                    meta.address().ok_or(crate::Error::MissingAddress)?,
                    encoded_path
                ))
            }
            _ => Err(crate::Error::UnsupportedFileType),
        }
    }

    fn decode(&self, _path: &str, filename: &str) -> Result<Box<dyn FileMeta>, crate::Error> {
        let parts: Vec<&str> = filename.split('-').collect();

        // 解析 root 文件
        if parts.len() == 2 {
            let suffix = parts[1];
            let file_type = match suffix {
                "phrase" => FileType::Phrase,
                "pk" => FileType::PrivateKey,
                "seed" => FileType::Seed,
                _ => return Err(crate::Error::UnsupportedFileType),
            };

            return Ok(Box::new(LegacyFileMeta {
                file_type,
                address: parts[0].to_string(),
                chain_code: None,
                derivation_path: None,
                // timestamp: Utc::now(),
            }));
        }

        // 解析 subs 文件（至少4部分：chain-addr-path-pk）
        if parts.len() >= 4 && parts.last() == Some(&"pk") {
            let chain = parts[0].to_string();
            let address = parts[1].to_string();

            // 合并中间部分作为编码路径（处理路径中包含-的情况）
            let encoded_path = parts[2..parts.len() - 1].join("-");

            let derivation_path =
                wallet_utils::parse_func::derivation_path_percent_decode(&encoded_path)?
                    .to_string();

            // let derivation_path = percent_decode_str(&encoded_path)
            //     .decode_utf8()?
            //     .into_owned();

            return Ok(Box::new(LegacyFileMeta {
                file_type: FileType::DerivedData,
                address,
                chain_code: Some(chain),
                derivation_path: Some(derivation_path),
                // timestamp: Utc::now(),
            }));
        }

        Err(crate::Error::FilenameInvalid)
    }

    fn version(&self) -> u32 {
        1
    }

    fn validate(&self, filename: &str) -> bool {
        let parts: Vec<&str> = filename.split('-').collect();

        // 检查基础分割格式
        if parts.len() < 2 {
            return false;
        }

        // 验证 Root 文件
        if parts.len() == 2 {
            return match parts[1] {
                "phrase" | "pk" | "seed" => true,
                _ => false,
            };
        }

        // 验证 Subs 文件
        if parts.len() >= 4 {
            let last_part = parts.last().unwrap();
            let has_valid_suffix = *last_part == "pk";

            // 检查路径编码格式（至少包含一个%）
            let encoded_path = parts[2..parts.len() - 1].join("-");
            let has_encoding = encoded_path.contains('%');

            return has_valid_suffix && has_encoding;
        }

        false
    }

    fn generate_filemeta(
        &self,
        file_type: FileType,
        address: &str,
        _account_index_map: Option<&wallet_utils::address::AccountIndexMap>,
        chain_code: Option<String>,
        derivation_path: Option<String>,
    ) -> Result<Box<dyn FileMeta>, crate::Error> {
        Ok(Box::new(LegacyFileMeta {
            file_type,
            address: address.to_string(),
            chain_code,
            derivation_path,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ADDRESS: &str = "0x668fb1D3Df02391064CEe50F6A3ffdbAEOCDb406";

    // 完整测试用例
    mod root_files {
        use super::*;

        #[test]
        fn parse_phrase_file() {
            let filename = format!("{}-phrase", TEST_ADDRESS);
            let meta = LegacyNaming.decode("", &filename).unwrap();

            assert_eq!(meta.file_type(), &FileType::Phrase);
            assert_eq!(meta.address(), Some(TEST_ADDRESS.to_string()));
            assert!(meta.chain_code().is_none());
        }

        #[test]
        fn parse_pk_file() {
            let filename = format!("{}-pk", TEST_ADDRESS);
            let meta = LegacyNaming.decode("", &filename).unwrap();

            assert_eq!(meta.file_type(), &FileType::PrivateKey);
            assert_eq!(meta.address(), Some(TEST_ADDRESS.to_string()));
        }
    }

    mod subs_files {
        use super::*;

        const SUBS_EXAMPLES: [(&str, &str, &str); 5] = [
            (
                "bnb-0x79276834D1c11039df3eFdE8204aA27CB661a0ff-m%2F44%27%2F60%27%2F0%27%2F0%2F0-pk",
                "bnb",
                "m/44'/60'/0'/0/0"
            ),
            (
                "btc-1Fykerj4TT5CuoXCUw6UxhS9ZrZync61Lf-m%2F44%27%2F0%27%2F0%27%2F0%2F0-pk",
                "btc",
                "m/44'/0'/0'/0/0"
            ),
            (
                "btc-bc1gr0wh89tnckm5g677aepcw37504qu5etf96gm20-m%2F84%27%2F0%27%2F0%27%2F0%2F0-pk",
                "btc",
                "m/84'/0'/0'/0/0"
            ),
            (
                "sol-33wDijijcuZwaBEGmgfMoFeyFbXphoCYmdNhHPwYJZ8e-m%2F44%27%2F501%27%2F0%27%2F0-pk",
                "sol",
                "m/44'/501'/0'/0"
            ),
            (
                "tron-TAqUJ9enU8KkZYySA51iQim7TxbbdLR2wn-m%2F44%27%2F195%27%2F0%27%2F0%2F0-pk",
                "tron",
                "m/44'/195'/0'/0/0"
            )
        ];

        #[test]
        fn parse_all_subs_formats() {
            let strategy = LegacyNaming;

            for (filename, expected_chain, expected_path) in SUBS_EXAMPLES {
                let meta = strategy.decode("", filename).unwrap();

                assert_eq!(meta.file_type(), &FileType::DerivedData);
                assert_eq!(meta.chain_code(), Some(expected_chain.to_string()));
                assert_eq!(meta.derivation_path(), Some(expected_path.to_string()));
            }
        }

        #[test]
        fn handle_special_characters() {
            let filename = "btc-特殊地址-m%2Ftest%2Fpath%2Fwith%2Fchinese-%E4%B8%AD%E6%96%87-pk";
            let meta = LegacyNaming.decode("", filename).unwrap();

            assert_eq!(
                meta.derivation_path(),
                Some("m/test/path/with/chinese-中文".to_string())
            );
        }
    }
}

#[cfg(test)]
mod validate_tests {
    use super::*;

    const VALID_ROOT: &str = "0x123abc-pk";
    const VALID_PHRASE: &str = "0x123abc-phrase";
    const VALID_SEED: &str = "0x123abc-seed";
    const VALID_SUB: &str = "eth-0x456def-m%2F44%27%2F0%27%2F0%27%2F0%2F0-pk";
    const INVALID_SUB: &str = "eth-0x456def-invalid_path-pk";

    #[test]
    fn validate_root_files() {
        let strategy = LegacyNaming;

        assert!(strategy.validate(VALID_ROOT));
        assert!(strategy.validate(VALID_PHRASE));
        assert!(strategy.validate(VALID_SEED));
        assert!(!strategy.validate("0x123-invalid"));
    }

    #[test]
    fn validate_subs_files() {
        let strategy = LegacyNaming;

        assert!(strategy.validate(VALID_SUB));
        assert!(!strategy.validate(INVALID_SUB)); // 缺少编码字符%
        assert!(!strategy.validate("chain-addr-pk")); // 部分不足
    }

    #[test]
    fn validate_edge_cases() {
        let strategy = LegacyNaming;

        // 正确包含多个连字符的路径
        assert!(strategy.validate("btc-addr-m%2Fpath%2Fwith-multiple-hyphens-pk"));

        // 无效的编码格式
        assert!(!strategy.validate("eth-addr-m/path/without/encoding-pk"));

        // 错误的后缀
        assert!(!strategy.validate("eth-addr-m%2Fpath-wrongsuffix"));
    }
}
