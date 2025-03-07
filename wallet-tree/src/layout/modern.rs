use std::{
    any,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::{
    layout::LayoutStrategy,
    naming::{
        modern::{ModernFileMetas, ModernNaming},
        FileMeta, FileType, NamingStrategy as _,
    },
    wallet_tree::{
        modern_adapter::{ModernRoot, ModernWalletBranch, ModernWalletTree},
        WalletTreeOps,
    },
};

// #[derive(Serialize, Deserialize)]
// pub struct DerivedKeys {
//     entries: BTreeMap<DerivationPath, KeyEntry>, // 按派生路径排序
// }

#[derive(Debug, Default, PartialEq, Clone, Serialize)]
pub struct ModernLayout;

impl LayoutStrategy for ModernLayout {
    fn resolve_path(&self, meta: Box<dyn FileMeta>) -> Result<PathBuf, crate::Error> {
        let base = PathBuf::from(&meta.address().ok_or(crate::Error::MissingAddress)?);

        match meta.file_type() {
            FileType::Phrase | FileType::PrivateKey | FileType::Seed => {
                Ok(base.join("root").join(self.generate_filename(meta)?))
            }
            FileType::DerivedData => {
                let idx = meta.account_index().ok_or(crate::Error::MissingIndex)?;
                Ok(base.join("subs").join(format!("key{}.keystore", idx)))
            }
            FileType::DerivedMeta => Ok(base.join("subs").join("derived_meta.json")),
            _ => Err(crate::Error::UnsupportedFileType),
        }
    }

    fn scan(&self, base_path: &Path) -> Result<Box<dyn WalletTreeOps>, crate::Error> {
        let mut entries = ModernWalletTree::default();
        // let address = base_path.file_name().unwrap().to_str().unwrap();

        for entry in wallet_utils::file_func::read_dir(base_path.to_path_buf())? {
            if let Ok(entry) = entry {
                let wallet_dir = entry.path();
                if wallet_dir.is_dir() {
                    let address = wallet_dir
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let root_dir = wallet_dir.join("root");
                    let subs_dir = wallet_dir.join("subs");

                    // 处理 root 文件
                    let mut wallet_branch = if root_dir.exists() {
                        let mut modern_root = ModernRoot::default();
                        for entry in std::fs::read_dir(root_dir).unwrap() {
                            let entry = entry.unwrap();
                            let path = entry.path();
                            if path.is_file() {
                                let filename = path.file_name().unwrap().to_str().unwrap();
                                tracing::info!("filename: {filename}");
                                let Ok(meta) = self
                                    .parse_meta_data(&path.to_string_lossy().to_string(), filename)
                                else {
                                    continue;
                                };

                                match meta.file_type() {
                                    FileType::PrivateKey => {
                                        modern_root.pk = Some(());
                                    }
                                    FileType::Phrase => {
                                        modern_root.phrase = Some(());
                                    }
                                    FileType::Seed => {
                                        modern_root.seed = Some(());
                                    }
                                    _ => continue,
                                }
                            }
                        }
                        ModernWalletBranch {
                            root: modern_root,
                            subs: Default::default(),
                        }
                    } else {
                        continue;
                    };

                    // 处理 subs 文件
                    if subs_dir.exists() {
                        // 加载元数据文件
                        let meta_file = subs_dir.join("derived_meta.json");
                        if meta_file.exists() {
                            let Ok(meta) = self
                                .parse_meta_data(&meta_file.to_string_lossy(), "derived_meta.json")
                            else {
                                continue;
                            };

                            // 加载所有密钥文件
                            if let ModernFileMetas::DerivedMeta(entries) = meta {
                                for mut entry in entries {
                                    let file_name = format!(
                                        "key{}.keystore",
                                        entry.account_index().ok_or(crate::Error::MissingIndex)?
                                    );
                                    let key_path = subs_dir.join(&file_name);

                                    if key_path.exists() {
                                        let Ok(meta) = self.parse_meta_data(
                                            &key_path.to_string_lossy(),
                                            &file_name,
                                        ) else {
                                            continue;
                                        };
                                        if let ModernFileMetas::DerivedData(_) = meta {
                                            entry.file_type = meta.file_type().clone();
                                            wallet_branch.subs.push(entry);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    entries.tree.insert(address, wallet_branch);
                } else {
                    continue;
                }
            }
        }

        Ok(Box::new(entries))
    }

    fn version(&self) -> u32 {
        2
    }
}

impl ModernLayout {
    fn parse_meta_data(&self, path: &str, filename: &str) -> Result<ModernFileMetas, crate::Error> {
        let meta_box: Box<dyn any::Any> = ModernNaming.decode(path, filename)?;
        meta_box
            .downcast::<ModernFileMetas>() // 直接调用，无需类型转换
            .map(|boxed| *boxed) // 解包 Box
            .map_err(|e| {
                tracing::info!("Failed to downcast: {:#?}", e);
                crate::Error::FailedToDowncast
            })
    }

    fn generate_filename(&self, meta: Box<dyn FileMeta>) -> Result<String, crate::Error> {
        ModernNaming.encode(meta)
    }
}
