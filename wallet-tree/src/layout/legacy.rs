use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{
    naming::{
        legacy::{LegacyFileMeta, LegacyNaming},
        FileMeta, FileType, NamingStrategy as _,
    },
    wallet_tree::{
        legecy_adapter::{LegacyWalletBranch, LegacyWalletTree},
        WalletTreeOps,
    },
};

use super::LayoutStrategy;

#[derive(Debug, Default, PartialEq, Clone, Serialize)]
pub struct LegacyLayout;

impl LayoutStrategy for LegacyLayout {
    fn resolve_path(&self, meta: Box<dyn FileMeta>) -> Result<PathBuf, crate::Error> {
        match meta.file_type() {
            FileType::Phrase | FileType::PrivateKey | FileType::Seed => {
                // Root 文件存储路径：{base}/{address}/root/{filename}
                Ok(
                    PathBuf::from(&meta.address().ok_or(crate::Error::MissingAddress)?)
                        .join("root")
                        .join(self.generate_filename(meta)?),
                )
            }
            FileType::DerivedData => {
                // Subs 文件存储路径：{base}/{address}/subs/{filename}
                Ok(
                    PathBuf::from(&meta.address().ok_or(crate::Error::MissingAddress)?)
                        .join("subs")
                        .join(self.generate_filename(meta)?),
                )
            }
            _ => Err(crate::Error::UnsupportedFileType),
        }
    }

    fn scan(&self, base_path: &Path) -> Result<Box<dyn WalletTreeOps>, crate::Error> {
        Ok(Box::new(LegacyWalletTree::traverse_directory_structure(
            &base_path.to_path_buf(),
        )?))
    }

    // fn scan(&self, base_path: &Path) -> Result<Box<dyn WalletTreeOps>, crate::Error> {
    //     let mut entries = LegacyWalletTree::default();

    //     // 扫描 root 目录
    //     for Ok(entry) in wallet_utils::file_func::read_dir(base_path.to_path_buf())? {
    //         let wallet_dir = entry.path();
    //         if wallet_dir.is_dir() {
    //             let address = wallet_dir
    //                 .file_name()
    //                 .unwrap()
    //                 .to_string_lossy()
    //                 .to_string();
    //             let root_dir = base_path.join("root");
    //             let subs_dir = base_path.join("subs");

    //             println!("root_dir: {root_dir:?}");
    //             let wallet_branch = if root_dir.exists() {
    //                 let modern_root = RootKeystoreInfo::default();
    //                 println!("存在");
    //                 for entry in std::fs::read_dir(root_dir).unwrap() {
    //                     let entry = entry.unwrap();
    //                     let path = entry.path();
    //                     if path.is_file() {
    //                         if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
    //                             let meta = self.parse_filename(filename)?;
    //                             entries.push(meta);
    //                         }
    //                     }
    //                 }

    //                 LegacyWalletBranch{
    //                     root_info: todo!(),
    //                     accounts: todo!(),
    //                 }
    //             } else {
    //                 continue;
    //             };

    //             // 扫描 subs 目录
    //             if subs_dir.exists() {
    //                 for entry in std::fs::read_dir(subs_dir).unwrap() {
    //                     let entry = entry.unwrap();
    //                     let path = entry.path();
    //                     if path.is_file() {
    //                         if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
    //                             let meta = self.parse_filename(filename)?;
    //                             entries.push(meta);
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     Ok(entries)
    // }

    fn version(&self) -> u32 {
        1 // 旧版策略版本号
    }
}

impl LegacyLayout {
    /// 根据文件名解析 FileMeta
    fn parse_filename(&self, filename: &str) -> Result<Box<dyn FileMeta>, crate::Error> {
        let naming = LegacyNaming; // 使用旧版命名策略
        naming.decode("", filename)
    }

    /// 生成文件名
    fn generate_filename(&self, meta: Box<dyn FileMeta>) -> Result<String, crate::Error> {
        let naming = LegacyNaming; // 使用旧版命名策略
        naming.encode(meta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    const TEST_ADDRESS: &str = "0x668fb1D3Df02391064CEe50F6A3ffdbAEOCDb406";

    #[test]
    fn test_resolve_root_path() {
        let layout = LegacyLayout;
        let meta = Box::new(LegacyFileMeta {
            file_type: FileType::PrivateKey,
            address: TEST_ADDRESS.to_string(),
            chain_code: None,
            derivation_path: None,
        });

        let path = layout.resolve_path(meta).unwrap();
        println!("path: {path:?}");
        assert_eq!(
            path,
            PathBuf::from(TEST_ADDRESS)
                .join("root")
                .join(format!("{}-pk", TEST_ADDRESS))
        );
    }

    #[test]
    fn test_resolve_subs_path() {
        let layout = LegacyLayout;
        let meta = Box::new(LegacyFileMeta {
            file_type: FileType::DerivedData,
            address: TEST_ADDRESS.to_string(),
            chain_code: Some("eth".to_string()),
            derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
        });

        let path = layout.resolve_path(meta).unwrap();
        assert_eq!(
            path,
            PathBuf::from(TEST_ADDRESS)
                .join("subs")
                .join("eth-0x668fb1D3Df02391064CEe50F6A3ffdbAEOCDb406-m%2F44%27%2F60%27%2F0%27%2F0%2F0-pk")
        );
    }

    #[test]
    fn test_scan_directory() {
        let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let base_path = dir
            .join("wallet_data")
            // .join("0x668fb1D3Df02391064CEe50F6A3ffdbAE0CDb406")
            ;
        // let base_path = PathBuf::from("./wallet_data/0x668fb1D3Df02391064CEe50F6A3ffdbAEOCDb406");
        let base_path = base_path.as_path();

        // 创建测试目录结构
        // let root_dir = base_path.join("root");
        // fs::create_dir_all(&root_dir).unwrap();
        // File::create(root_dir.join(format!("{}-pk", TEST_ADDRESS))).unwrap();

        // let subs_dir = base_path.join("subs");
        // fs::create_dir_all(&subs_dir).unwrap();
        // File::create(subs_dir.join("eth-0x123-m%2F44%27%2F60%27%2F0%27%2F0%2F0-pk")).unwrap();

        // 扫描目录
        let layout = LegacyLayout;
        let entries = layout.scan(base_path).unwrap();

        println!("entries: {entries:#?}");
        // assert_eq!(entries.len(), 11);
        // assert!(entries
        //     .iter()
        //     .any(|m| m.file_type() == &FileType::PrivateKey));
        // assert!(entries
        //     .iter()
        //     .any(|m| m.file_type() == &FileType::DerivedData));
    }
}
