// mod meta;
pub(crate) mod legacy;
pub(crate) mod modern;
// mod v1;
// mod v2;

use std::path::{Path, PathBuf};

use crate::{naming::FileMeta, wallet_tree::WalletTreeOps};

pub trait LayoutStrategy: Send + Sync {
    /// 获取文件存储路径
    fn resolve_path(&self, meta: Box<dyn FileMeta>) -> Result<PathBuf, crate::Error>;

    /// 遍历目录获取所有文件元数据
    fn scan(&self, base_path: &Path) -> Result<Box<dyn WalletTreeOps>, crate::Error>;

    /// 策略版本号
    fn version(&self) -> u32;
}
