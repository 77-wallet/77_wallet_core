pub(crate) mod legacy;
pub trait NamingStrategy: Send + Sync {
    /// 将元数据编码为文件名
    fn encode(&self, meta: &FileMeta) -> Result<String, crate::Error>;

    /// 从文件名解析元数据
    fn decode(&self, filename: &str) -> Result<FileMeta, crate::Error>;

    /// 验证文件名格式
    fn validate(&self, filename: &str) -> bool;

    /// 策略版本号
    fn version(&self) -> u32;
}

#[derive(Debug, Clone)]
pub struct FileMeta {
    // pub directory_naming: DirectoryNaming,
    pub file_type: FileType,
    pub address: String,
    pub chain_code: Option<String>,
    pub derivation_path: Option<String>,
    // pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum DirectoryNaming {
    Root,
    Subs,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FileType {
    PrivateKey,
    Phrase,
    Seed,
    DerivedKey,
}

impl FileType {
    pub fn to_string(&self) -> String {
        match self {
            FileType::PrivateKey => "pk",
            FileType::Phrase => "phrase",
            FileType::Seed => "seed",
            FileType::DerivedKey => "pk",
        }
        .to_string()
    }
}
