pub mod legecy_adapter;
pub mod modern_adapter;

use serde::{Deserialize, Serialize};
use wallet_types::chain::chain::ChainCode;

use crate::{
    io::IoStrategy,
    layout::LayoutStrategy,
    naming::{FileMeta, NamingStrategy},
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WalletTreeStrategy {
    V1,
    V2,
}

impl WalletTreeStrategy {
    pub fn get_wallet_tree(
        self,
        wallet_dir: &std::path::PathBuf,
    ) -> Result<Box<dyn WalletTreeOps>, crate::Error> {
        Ok(match self {
            WalletTreeStrategy::V1 => Box::new(
                legecy_adapter::LegacyWalletTree::traverse_directory_structure(wallet_dir)?,
            ),
            WalletTreeStrategy::V2 => {
                Box::new(modern_adapter::ModernWalletTree::traverse(wallet_dir)?)
            }
        })
    }
}

pub trait WalletTreeOps: std::any::Any + std::fmt::Debug {
    fn layout(&self) -> Box<dyn LayoutStrategy>;
    fn naming(&self) -> Box<dyn NamingStrategy>;
    fn io(&self) -> Box<dyn IoStrategy>;

    fn traverse(wallet_dir: &std::path::PathBuf) -> Result<Self, crate::Error>
    where
        Self: Sized;
    // fn deprecate_subkeys(
    //     &mut self,
    //     wallet_address: &str,
    //     subs_path: std::path::PathBuf,
    // ) -> Result<(), crate::Error>;

    fn delete_subkey(
        &mut self,
        // naming: Box<dyn NamingStrategy>,
        wallet_address: &str,
        address: &str,
        chain_code: &str,
        file_path: &dyn AsRef<std::path::Path>,
        password: &str,
    ) -> Result<(), crate::Error>;

    // fn delete_subkeys(
    //     &mut self,
    //     wallet_address: &str,
    //     subs_path: &std::path::PathBuf,
    //     address: &str,
    //     chain: &ChainCode,
    // ) -> Result<(), crate::Error>;

    fn get_wallet_branch(
        &self,
        wallet_address: &str,
    ) -> Result<Box<dyn WalletBranchOps>, crate::Error>;

    // fn get_mut_wallet_branch(
    //     &mut self,
    //     wallet_address: &str,
    // ) -> Result<Box<&mut dyn WalletBranchOps>, crate::Error>;

    // fn iter(&self) -> Box<dyn Iterator<Item = (String, Box<dyn WalletBranchOps>)>>;
    fn iter(
        &self,
        // ) -> impl Iterator<Item = (&String, &dyn WalletBranchOps)>;
    ) -> Box<dyn Iterator<Item = (&String, &dyn WalletBranchOps)> + '_>;
}

pub trait WalletBranchOps {
    fn get_root(&self) -> Box<dyn RootTrait>;
    fn get_account(&self, address: &str, chain_code: &ChainCode) -> Option<Box<dyn AccountTrait>>;

    fn get_accounts(&self) -> Vec<Box<dyn AccountTrait>>;
}

pub trait AccountTrait: FileMeta + std::fmt::Debug + Send + Sync {
    // 定义公共接口方法
    fn get_address(&self) -> &str;
    fn get_filemeta(&self) -> Box<dyn crate::naming::FileMeta>;
}

pub trait RootTrait: std::fmt::Debug + Send + Sync {
    fn get_address(&self) -> &str;
    fn get_phrase_filemeta(&self) -> Option<()>;
    fn get_pk_filemeta(&self) -> Option<()>;
    fn get_seed_filemeta(&self) -> Option<()>;
}
