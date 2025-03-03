use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WalletMeta {
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub layout_version: u32,
    pub naming_version: u32,
}

impl WalletMeta {
    pub fn read(base_path: &Path) -> Result<Self, crate::Error> {
        let path = base_path.join(".walletmeta");
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn write(&self, base_path: &Path) -> Result<(), crate::Error> {
        let path = base_path.join(".walletmeta");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
