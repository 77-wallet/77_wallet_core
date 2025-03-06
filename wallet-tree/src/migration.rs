use std::{fs, path::PathBuf};

use crate::{layout::LayoutStrategy, naming::NamingStrategy};

pub struct MigrationEngine<S, D>
where
    D: LayoutStrategy,
{
    src_strategy: S,
    dst_strategy: D,
    dry_run: bool,
    backup_dir: PathBuf,
}

impl<S, D> MigrationEngine<S, D>
where
    S: NamingStrategy,
    D: LayoutStrategy,
{
    pub fn migrate_file(&self, old_path: PathBuf) -> Result<(), crate::Error> {
        // 步骤1：解析旧文件
        let old_meta = self.src_strategy.decode(
            &old_path.to_string_lossy().to_string(),
            old_path.file_name().unwrap().to_str().unwrap(),
        )?;

        // 步骤2：生成新路径
        let new_path = self.dst_strategy.resolve_path(old_meta).unwrap();

        // 步骤3：创建备份
        let backup_path = self.backup_dir.join(old_path.file_name().unwrap());
        fs::copy(&old_path, &backup_path).unwrap();

        // 步骤4：迁移文件
        if !self.dry_run {
            fs::create_dir_all(new_path.parent().unwrap()).unwrap();
            fs::rename(&old_path, &new_path).unwrap();
        }

        // 步骤5：记录审计日志
        // self.audit_log
        //     .log_migration(&old_path, &new_path, &old_meta)?;

        Ok(())
    }
}
