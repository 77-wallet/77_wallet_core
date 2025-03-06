use std::{
    fs,
    ops::{Deref, DerefMut},
};

use serde::Serialize;
use wallet_keystore::KeystoreBuilder;
use wallet_types::chain::chain::ChainCode;

use crate::{
    layout::LayoutStrategy,
    naming::{
        modern::{DerivedMetadata, KeyMeta, KeystoreData, ModernFileMeta},
        FileMeta,
    },
};

use super::{AccountTrait, RootTrait, WalletBranchOps, WalletTreeOps};

#[derive(Clone, Debug, Default)]
pub struct ModernWalletTree {
    pub layout: crate::layout::modern::ModernLayout,
    pub naming: crate::naming::modern::ModernNaming,
    pub io: crate::io::modern::ModernIo,
    pub tree: ModernWalletBranches,
}

#[derive(Clone, Debug, Default)]
pub struct ModernWalletBranches(std::collections::HashMap<String, ModernWalletBranch>);

impl Deref for ModernWalletBranches {
    type Target = std::collections::HashMap<String, ModernWalletBranch>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ModernWalletBranches {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ModernWalletTree {}

impl WalletTreeOps for ModernWalletTree {
    fn layout(&self) -> Box<dyn crate::layout::LayoutStrategy> {
        Box::new(self.layout.clone())
    }

    fn naming(&self) -> Box<dyn crate::naming::NamingStrategy> {
        Box::new(self.naming.clone())
    }

    fn io(&self) -> Box<dyn crate::io::IoStrategy> {
        Box::new(self.io.clone())
    }

    fn traverse(wallet_dir: &std::path::PathBuf) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        let wallet_tree: Box<dyn std::any::Any> =
            crate::layout::modern::ModernLayout.scan(&wallet_dir)?;
        wallet_tree
            .downcast::<ModernWalletTree>() // 直接调用，无需类型转换
            .map(|boxed| *boxed) // 解包 Box
            .map_err(|_| crate::Error::FailedToDowncast)
    }

    // fn deprecate_subkeys(
    //     &mut self,
    //     wallet_address: &str,
    //     subs_path: std::path::PathBuf,
    // ) -> Result<(), crate::Error> {
    //     todo!()
    // }

    fn delete_subkey(
        &mut self,
        // naming: Box<dyn crate::naming::NamingStrategy>,
        wallet_address: &str,
        address: &str,
        chain_code: &str,
        file_path: &dyn AsRef<std::path::Path>,
        password: &str,
    ) -> Result<(), crate::Error> {
        let base_path = file_path.as_ref();
        let meta_path = base_path.join("derived_meta.json");
        let subs_dir = base_path;

        // 1. 处理元数据文件
        let mut metadata: DerivedMetadata = if meta_path.exists() {
            let content = fs::read_to_string(&meta_path).unwrap();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            return Err(crate::Error::MetadataNotFound);
        };
        tracing::info!("delete_subkey =============== 1");

        // 查找需要删除的条目并记录关联的密钥文件
        let mut keys_to_delete = Vec::new();
        for (account_idx, key_metas) in &mut metadata.accounts {
            // 保留不匹配的条目
            key_metas.retain(|meta| {
                let should_keep = !(meta.address == address && meta.chain_code == chain_code);
                if !should_keep {
                    // 生成要删除的key标识
                    let key = KeyMeta {
                        chain_code: meta.chain_code.clone(),
                        address: meta.address.clone(),
                        derivation_path: meta.derivation_path.clone(),
                    };
                    keys_to_delete.push((*account_idx, key.encode()));
                }
                should_keep
            });
        }

        tracing::info!("delete_subkey =============== 2");
        // 删除空账户
        metadata.accounts.retain(|_, metas| !metas.is_empty());

        // 原子写入新元数据
        let temp_meta_path = meta_path.with_extension("tmp");
        wallet_utils::file_func::write_all(
            &temp_meta_path,
            &serde_json::to_vec_pretty(&metadata).unwrap(),
        )?;
        fs::rename(&temp_meta_path, &meta_path).unwrap();

        tracing::info!("delete_subkey =============== 3");
        tracing::info!("metadata: {metadata:#?}");
        // 2. 处理密钥文件
        for (account_idx, encoded_key) in keys_to_delete {
            tracing::info!("account_idx: {account_idx:#?}");
            let key_filename = format!("key{}.keystore", account_idx);
            let file_path = subs_dir.join(&key_filename);
            tracing::info!("data_path: {file_path:#?}");

            if !file_path.exists() {
                continue;
            }

            // 加载并解密数据
            let keystore = KeystoreBuilder::new_decrypt(&file_path, password).load()?;

            // 转换为可操作结构
            let mut keystore_data: KeystoreData = keystore.try_into()?;

            tracing::warn!("keystore_data 1: {keystore_data:#?}");
            // 删除目标条目
            keystore_data.remove(&encoded_key);

            tracing::warn!("keystore_data 2: {keystore_data:#?}");
            // 决定是否保留文件
            // if keystore_data.get(&account_idx).is_none(){

            // }
            if keystore_data.is_empty() {
                fs::remove_file(&file_path).unwrap();
            } else {
                // 重新加密保存
                let rng = rand::thread_rng();
                tracing::info!("data_path: {file_path:#?}");
                KeystoreBuilder::new_encrypt(
                    &subs_dir,
                    password,
                    &wallet_utils::serde_func::serde_to_vec(&keystore_data)?,
                    rng,
                    wallet_keystore::KdfAlgorithm::Argon2id,
                    &key_filename,
                )
                .save()?;
            }
        }

        Ok(())
    }

    fn get_wallet_branch(
        &self,
        wallet_address: &str,
    ) -> Result<Box<dyn WalletBranchOps>, crate::Error> {
        self.tree
            .get(wallet_address)
            .map(|branch| Box::new(branch.clone()) as Box<dyn WalletBranchOps>)
            .ok_or(crate::Error::LocalNoWallet)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&String, &dyn WalletBranchOps)> + '_> {
        Box::new(
            self.tree
                .iter()
                .map(|(k, v)| (k, v as &dyn WalletBranchOps)),
        )
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModernWalletBranch {
    // 根账户信息
    pub root: ModernRoot,
    pub subs: Vec<ModernFileMeta>,
}

impl ModernWalletBranch {
    pub fn delete_subkey(
        &mut self,
        wallet_address: &str,
        subs_path: &std::path::PathBuf,
        address: &str,
        chain_code: &ChainCode,
    ) -> Result<(), crate::Error> {
        let index = self.subs.iter().position(|sub| {
            sub.get_address() == address && sub.chain_code() == Some(chain_code.to_string())
        });

        Ok(())
    }
}

impl WalletBranchOps for ModernWalletBranch {
    fn get_root(&self) -> Box<dyn super::RootTrait> {
        Box::new(self.root.clone())
    }

    fn get_account(
        &self,
        address: &str,
        chain_code: &wallet_types::chain::chain::ChainCode,
    ) -> Option<Box<dyn super::AccountTrait>> {
        self.subs
            .iter()
            .find(|sub| {
                sub.get_address() == address && sub.chain_code() == Some(chain_code.to_string())
            })
            .map(|sub| Box::new(sub.clone()) as Box<dyn AccountTrait>)
    }

    fn get_accounts(&self) -> Vec<Box<dyn super::AccountTrait>> {
        self.subs
            .iter()
            .map(|sub| Box::new(sub.clone()) as Box<dyn AccountTrait>)
            .collect()
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModernRoot {
    // 根账户信息
    pub address: String,
    pub phrase: Option<()>,
    pub pk: Option<()>,
    pub seed: Option<()>,
}

impl RootTrait for ModernRoot {
    fn get_address(&self) -> &str {
        &self.address
    }

    fn get_phrase_filemeta(&self) -> Option<()> {
        self.phrase
    }

    fn get_pk_filemeta(&self) -> Option<()> {
        self.pk
    }

    fn get_seed_filemeta(&self) -> Option<()> {
        self.seed
    }
}

impl AccountTrait for ModernFileMeta {
    fn get_address(&self) -> &str {
        let t = self.address.as_deref();
        t.unwrap_or_default()
    }

    fn get_filemeta(&self) -> Box<dyn crate::naming::FileMeta> {
        Box::new(self.clone())
    }
}
