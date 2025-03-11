use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use wallet_keystore::RecoverableData;
use wallet_utils::address::AccountIndexMap;

use super::{FileMeta, FileType, NamingStrategy};
use crate::error::Error;

#[derive(Clone, Debug)]
pub enum ModernFileMetas {
    Phrase(ModernFileMeta),
    PrivateKey(ModernFileMeta),
    Seed(ModernFileMeta),
    DerivedData(u32),
    DerivedMeta(Vec<ModernFileMeta>),
}

// #[derive(Clone, Debug)]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct ModernFileMeta {
    pub file_type: FileType,
    pub address: Option<String>,
    pub account_index_map: Option<wallet_utils::address::AccountIndexMap>,
    pub chain_code: Option<String>,
    pub derivation_path: Option<String>,
}

impl FileMeta for ModernFileMeta {
    fn file_type(&self) -> &FileType {
        &self.file_type
    }
    fn address(&self) -> Option<String> {
        self.address.clone()
    }
    fn account_index(&self) -> Option<u32> {
        self.account_index_map.as_ref().map(|a| a.account_id)
    }
    fn chain_code(&self) -> Option<String> {
        self.chain_code.clone()
    }
    fn derivation_path(&self) -> Option<String> {
        self.derivation_path.clone()
    }
}

impl FileMeta for ModernFileMetas {
    fn file_type(&self) -> &FileType {
        match self {
            ModernFileMetas::Phrase(modern_file_meta) => &modern_file_meta.file_type,
            ModernFileMetas::PrivateKey(modern_file_meta) => &modern_file_meta.file_type,
            ModernFileMetas::Seed(modern_file_meta) => &modern_file_meta.file_type,
            ModernFileMetas::DerivedData(_) => &FileType::DerivedData,
            ModernFileMetas::DerivedMeta(_) => &FileType::DerivedMeta,
        }
    }

    fn address(&self) -> Option<String> {
        match self {
            ModernFileMetas::Phrase(modern_file_meta) => modern_file_meta.address.clone(),
            ModernFileMetas::PrivateKey(modern_file_meta) => modern_file_meta.address.clone(),
            ModernFileMetas::Seed(modern_file_meta) => modern_file_meta.address.clone(),
            ModernFileMetas::DerivedData(_) => None,
            ModernFileMetas::DerivedMeta(_) => None,
        }
    }

    fn account_index(&self) -> Option<u32> {
        match self {
            ModernFileMetas::Phrase(_) => None,
            ModernFileMetas::PrivateKey(_) => None,
            ModernFileMetas::Seed(_) => None,
            ModernFileMetas::DerivedData(index) => Some(*index),
            ModernFileMetas::DerivedMeta(_) => None,
        }
    }

    fn chain_code(&self) -> Option<String> {
        match self {
            ModernFileMetas::Phrase(modern_file_meta) => modern_file_meta.chain_code.clone(),
            ModernFileMetas::PrivateKey(modern_file_meta) => modern_file_meta.chain_code.clone(),
            ModernFileMetas::Seed(modern_file_meta) => modern_file_meta.chain_code.clone(),
            ModernFileMetas::DerivedData(_) => None,
            ModernFileMetas::DerivedMeta(_) => None,
        }
    }

    fn derivation_path(&self) -> Option<String> {
        match self {
            ModernFileMetas::Phrase(modern_file_meta) => modern_file_meta.derivation_path.clone(),
            ModernFileMetas::PrivateKey(modern_file_meta) => {
                modern_file_meta.derivation_path.clone()
            }
            ModernFileMetas::Seed(modern_file_meta) => modern_file_meta.derivation_path.clone(),
            ModernFileMetas::DerivedData(_) => None,
            ModernFileMetas::DerivedMeta(_) => None,
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct ModernNaming;

impl NamingStrategy for ModernNaming {
    fn encode(&self, meta: Box<dyn FileMeta>) -> Result<String, Error> {
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
                let idx = meta.account_index();
                Ok(format!(
                    "key{}.keystore",
                    idx.ok_or(crate::Error::MissingIndex)?
                ))
            }
            FileType::DerivedMeta => Ok("derived_meta.json".to_string()),
            _ => Err(Error::UnsupportedFileType),
        }
    }

    fn decode(&self, path: &str, filename: &str) -> Result<Box<dyn FileMeta>, Error> {
        if let Some(idx) = filename
            .strip_prefix("key")
            .and_then(|s| s.strip_suffix(".keystore"))
            .and_then(|s| s.parse::<u32>().ok())
        {
            return Ok(Box::new(ModernFileMetas::DerivedData(idx)));
        }

        // let parts: Vec<&str> = filename.split('-').collect();
        match filename {
            "derived_meta.json" => {
                let content = std::fs::read_to_string(path).unwrap();
                let metadata: DerivedMetadata = serde_json::from_str(&content).unwrap();

                let mut metas = Vec::new();
                for (index, _v) in metadata.accounts.into_iter() {
                    let m = ModernFileMeta {
                        file_type: FileType::DerivedMeta,
                        address: None,
                        account_index_map: Some(AccountIndexMap::from_account_id(index)?),
                        chain_code: None,
                        derivation_path: None,
                    };
                    metas.push(m);
                }

                Ok(Box::new(ModernFileMetas::DerivedMeta(metas)))
            }
            // _ => {
            //     let suffix = parts[1];
            //     let file_type = match suffix {
            //         "phrase" => FileType::Phrase,
            //         "pk" => FileType::PrivateKey,
            //         "seed" => FileType::Seed,
            //         _ => return Err(Error::UnsupportedFileType),
            //     };
            //     let meta = ModernFileMeta {
            //         file_type: file_type.clone(),
            //         account_index_map: None,
            //         address: Some(parts[0].to_string()),
            //         chain_code: Default::default(),
            //         derivation_path: Default::default(),
            //     };
            //     Ok(Box::new(match file_type {
            //         FileType::PrivateKey => ModernFileMetas::PrivateKey(meta),
            //         FileType::Phrase => ModernFileMetas::Phrase(meta),
            //         FileType::Seed => ModernFileMetas::Seed(meta),
            //         _ => return Err(Error::FilenameInvalid),
            //     }))
            // }
            _ => Err(Error::FilenameInvalid),
        }
    }

    fn version(&self) -> u32 {
        2
    }

    fn validate(&self, filename: &str) -> bool {
        filename.ends_with("-phrase")
            || filename.ends_with("-pk")
            || filename.ends_with("-seed")
            || filename == "derived_keys.keystore"
            || filename == "derived_meta.json"
    }

    fn generate_filemeta(
        &self,
        file_type: FileType,
        address: &str,
        account_index_map: Option<&wallet_utils::address::AccountIndexMap>,
        chain_code: Option<String>,
        derivation_path: Option<String>,
    ) -> Result<Box<dyn FileMeta>, crate::Error> {
        Ok(Box::new(ModernFileMeta {
            file_type,
            address: Some(address.to_string()),
            account_index_map: account_index_map.cloned(),
            chain_code: chain_code,
            derivation_path: derivation_path,
        }))
    }
}

// #[derive(Serialize, Deserialize)]
// pub struct DerivedData {
//     pub entries: std::collections::HashMap<KeyMeta>,
// }

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DerivedMetadata {
    pub accounts: std::collections::BTreeMap<u32, KeyMetas>, // 按索引组织的账户
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct KeyMetas(Vec<KeyMeta>);

impl Deref for KeyMetas {
    type Target = Vec<KeyMeta>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KeyMetas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct KeystoreData(std::collections::BTreeMap<String, Vec<u8>>);

impl TryFrom<RecoverableData> for KeystoreData {
    type Error = crate::Error;

    fn try_from(value: RecoverableData) -> Result<Self, Self::Error> {
        // let s = value.into_string()?;

        // let val =
        // wallet_utils::serde_func::serde_to_value(value.inner())?;
        // tracing::warn!("val: {val:?}");
        // todo!()
        Ok(wallet_utils::serde_func::serde_from_slice(&value.inner())?)
    }
}

impl Deref for KeystoreData {
    type Target = std::collections::BTreeMap<String, Vec<u8>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KeystoreData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyMeta {
    pub chain_code: String,
    pub address: String,
    pub derivation_path: String,
}
const DELIMITER: &str = "::";

impl KeyMeta {
    pub fn encode(&self) -> String {
        format!(
            "{}{}{}{}{}",
            escape_str(&self.chain_code),
            DELIMITER,
            escape_str(&self.address),
            DELIMITER,
            escape_str(&self.derivation_path)
        )
    }

    pub fn decode(s: &str) -> Result<Self, crate::Error> {
        let parts: Vec<&str> = s.split(DELIMITER).collect();
        if parts.len() != 3 {
            return Err(crate::Error::InvalidKeyFormat);
        }

        Ok(Self {
            chain_code: unescape_str(parts[0])?,
            address: unescape_str(parts[1])?,
            derivation_path: unescape_str(parts[2])?,
        })
    }
}

// 转义规则：
// 原字符 | 转义后
//   :    | \c
//   \    | \\
fn escape_str(s: &str) -> String {
    s.replace('\\', r"\\").replace(':', r"\c")
}

fn unescape_str(s: &str) -> Result<String, crate::Error> {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => result.push('\\'),
                Some('c') => result.push(':'),
                _ => return Err(crate::Error::InvalidEscapeSequence),
            }
        } else {
            result.push(c);
        }
    }
    Ok(result)
}
