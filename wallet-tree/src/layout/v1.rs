use std::path::Path;

use super::LayoutStrategy;

pub struct LegacyLayout;

impl LayoutStrategy for LegacyLayout {
    fn version(&self) -> u32 {
        1
    }

    fn init_layout(&self, base_path: &Path) -> Result<(), crate::Error> {
        create_dir(base_path.join("root"))?;
        create_dir(base_path.join("subs"))?;
        Ok(())
    }

    fn resolve_path(&self, base_path: &Path, key: &FileKey) -> PathBuf {
        match key.file_type {
            FileType::Root => base_path.join("root"),
            FileType::Sub => base_path.join("subs"),
            _ => base_path.join("misc"),
        }
    }

    fn parse_layout(&self, base_path: &Path) -> Result<WalletTree, crate::Error> {
        // 原traverse_directory_structure逻辑
    }
    
    fn migrate_from(&self, prev: &dyn LayoutStrategy) -> Result<MigrationPlan, crate::Error> {
        todo!()
    }
}
