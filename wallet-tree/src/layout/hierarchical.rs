use std::path::PathBuf;

use crate::{
    layout::LayoutStrategy,
    naming::{FileMeta, FileType},
};

struct HierarchicalLayout;

impl LayoutStrategy for HierarchicalLayout {
    fn resolve_path(&self, meta: &FileMeta) -> Result<PathBuf, crate::Error> {
        Ok(match meta.file_type {
            FileType::PrivateKey | FileType::Phrase | FileType::Seed => {
                // 示例：wallets/{prefix_2}/{address}/key.pk
                let prefix = &meta.address[0..2];
                PathBuf::from("wallets").join(prefix).join(&meta.address)
            }
            FileType::DerivedKey => {
                // 示例：derived/{chain}/m/44'/60'/0'/0/0/key.pk
                let chain = meta.chain_code.as_deref().unwrap_or("unknown");
                let path = meta.derivation_path.as_deref().unwrap_or("");
                PathBuf::from("derived")
                    .join(chain)
                    .join(path.trim_start_matches('m'))
                    .join("key.pk")
            }
            _ => PathBuf::from("others"),
        })
    }

    fn scan(
        &self,
        base_path: &std::path::Path,
    ) -> Result<Vec<crate::naming::FileMeta>, crate::Error> {
        todo!()
    }

    fn version(&self) -> u32 {
        todo!()
    }
}
