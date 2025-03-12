use serde::Serialize;

pub(crate) mod legacy;
pub(crate) mod modern;
pub trait NamingStrategy: Send + Sync {
    /// 将元数据编码为文件名
    fn encode(&self, meta: Box<dyn FileMeta>) -> Result<String, crate::Error>;

    /// 从文件名解析元数据
    fn decode(&self, path: &str, filename: &str) -> Result<Box<dyn FileMeta>, crate::Error>;

    /// 验证文件名格式
    fn validate(&self, filename: &str) -> bool;

    /// 策略版本号
    fn version(&self) -> u32;

    /// 生成元数据
    fn generate_filemeta(
        &self,
        file_type: FileType,
        address: &str,
        account_index_map: Option<&wallet_utils::address::AccountIndexMap>,
        chain_code: Option<String>,
        derivation_path: Option<String>,
    ) -> Result<Box<dyn FileMeta>, crate::Error>;
}

pub trait FileMeta: std::any::Any + Send + Sync {
    fn file_type(&self) -> &FileType;
    fn account_index(&self) -> Option<u32>;
    fn address(&self) -> Option<String>;
    fn chain_code(&self) -> Option<String>;
    fn derivation_path(&self) -> Option<String>;
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum FileType {
    Root,
    PrivateKey,
    Phrase,
    Seed,
    DerivedData,
    DerivedMeta,
    DeprecatedPk,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct FileTypes(Vec<FileType>);

#[derive(Debug, Clone)]
pub enum DirectoryNaming {
    Root,
    Subs,
}

// impl FileType {
//     pub fn to_string(&self) -> String {
//         match self {
//             FileType::PrivateKey => "pk",
//             FileType::Phrase => "phrase",
//             FileType::Seed => "seed",
//             FileType::DerivedData => "pk",
//             FileType::DerivedMeta => "derived_meta.json",
//         }
//         .to_string()
//     }
// }
