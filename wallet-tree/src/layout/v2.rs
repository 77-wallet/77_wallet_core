pub struct HierarchicalLayout;

impl LayoutStrategy for HierarchicalLayout {
    fn version(&self) -> u32 { 2 }
    
    fn init_layout(&self, base_path: &Path) -> Result<()> {
        create_dir(base_path.join("accounts"))?;
        create_dir(base_path.join("metadata"))?;
        create_dir(base_path.join("transactions"))?;
        Ok(())
    }
    
    fn resolve_path(&self, base_path: &Path, key: &FileKey) -> PathBuf {
        match key.file_type {
            FileType::Root => base_path.join("accounts/root"),
            FileType::Sub => base_path.join("accounts/derived"),
            FileType::Phrase => base_path.join("metadata/phrases"),
            FileType::Seed => base_path.join("metadata/seeds"),
            _ => base_path.join("misc"),
        }
    }
}