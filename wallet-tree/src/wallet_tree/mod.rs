pub mod root;
pub mod subs;
use root::RootKeystoreInfo;
use serde::Serialize;
use subs::SubsKeystoreInfo;
use wallet_types::chain::chain::ChainCode;

/// 钱包
///       根              子
///    pk    seed       pk  pk
///                      
/// 表示钱包的目录结构，将钱包名称映射到其下的账户目录结构。
#[derive(Debug, Default, PartialEq, Clone, Serialize)]
pub struct WalletTree {
    pub tree: std::collections::HashMap<String, WalletBranch>,
}

impl WalletTree {
    pub fn deprecate_subkeys(
        mut self,
        wallet_address: alloy::primitives::Address,
        subs_path: std::path::PathBuf,
    ) -> Result<(), crate::Error> {
        let wallet = self.get_mut_wallet_branch(&wallet_address.to_string())?;

        wallet.deprecate_subkeys(&wallet_address.to_string(), subs_path)?;
        Ok(())
    }

    pub fn delete_subkeys(
        &mut self,
        wallet_address: &str,
        subs_path: &std::path::PathBuf,
        address: &str,
        chain_code: &ChainCode,
    ) -> Result<(), crate::Error> {
        let wallet = self.get_mut_wallet_branch(wallet_address)?;
        wallet.delete_subkey(wallet_address, subs_path, address, chain_code)?;
        Ok(())
    }

    pub fn get_wallet_branch(&self, wallet_address: &str) -> Result<&WalletBranch, crate::Error> {
        self.tree
            .get(wallet_address)
            .ok_or(super::error::Error::LocalNoWallet)
    }

    pub fn get_mut_wallet_branch(
        &mut self,
        wallet_address: &str,
    ) -> Result<&mut WalletBranch, crate::Error> {
        self.tree
            .get_mut(wallet_address)
            .ok_or(super::error::Error::LocalNoWallet)
    }

    /// 遍历指定目录结构，并将结果映射到数据结构中。
    ///
    /// # Arguments
    ///
    /// * `base_path` - 基础目录路径。
    ///
    /// # Returns
    ///
    /// 返回表示目录结构的数据结构，将钱包名称映射到其下的账户目录结构。
    ///
    /// # Example
    ///
    /// ```no_run
    /// let base_path = PathBuf::from("/path/to/wallets");
    /// let structure = traverse_directory_structure(base_path);
    /// ```
    pub fn traverse_directory_structure(
        wallet_dir: &std::path::PathBuf,
    ) -> Result<WalletTree, crate::Error> {
        let mut wallet_tree = WalletTree::default();

        let root = wallet_dir;
        for entry in wallet_utils::file_func::read_dir(root)? {
            let mut wallet_branch = WalletBranch::default();
            let entry = entry.map_err(|e| crate::Error::Utils(e.into()))?;
            let path = entry.path();

            if path.is_dir() {
                let wallet_name = path.file_name().unwrap().to_string_lossy().to_string();
                let root_dir = path.join("root");
                let subs_dir = path.join("subs");

                wallet_utils::file_func::create_dir_all(&root_dir)?;
                wallet_utils::file_func::create_dir_all(&subs_dir)?;

                let Some(root_dir) = wallet_utils::file_func::read_dir(root_dir)?
                    .filter_map(Result::ok)
                    .map(|e| e.file_name())
                    .find(|e| e.to_string_lossy().ends_with("-pk"))
                else {
                    continue;
                };

                let pk_filename = root_dir.to_string_lossy().to_string();

                wallet_branch.add_root_from_filename(&pk_filename)?;

                for subs_entry in wallet_utils::file_func::read_dir(subs_dir)? {
                    let subs_entry = subs_entry.map_err(|e| crate::Error::Utils(e.into()))?;
                    let subs_path = subs_entry.path();

                    if subs_path.is_file()
                        && subs_path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .ends_with("-pk")
                    {
                        if let Err(e) = wallet_branch.add_subkey_from_filename(
                            &subs_path.file_name().unwrap().to_string_lossy(),
                        ) {
                            tracing::error!("[traverse_directory_structure] subs error: {e}");
                            continue;
                        };
                    }
                }

                // 将钱包分支添加到钱包树中
                wallet_tree
                    .tree
                    .insert(wallet_name.to_string(), wallet_branch);
            }
        }

        Ok(wallet_tree)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletBranch {
    // 根账户信息
    pub root_info: RootKeystoreInfo,
    pub accounts: Vec<SubsKeystoreInfo>,
}

impl Default for WalletBranch {
    fn default() -> Self {
        Self {
            root_info: RootKeystoreInfo {
                address: Default::default(),
                suffix: crate::utils::file::Suffix::Pk { deprecated: false },
            },
            accounts: Default::default(),
        }
    }
}

// pub struct
impl WalletBranch {
    // 根据文件名解析并添加密钥
    pub fn add_subkey_from_filename(&mut self, filename: &str) -> Result<(), crate::Error> {
        let wallet_info =
            crate::utils::file::extract_sub_address_and_derive_path_from_filename(filename)?;
        self.accounts.push(wallet_info);

        Ok(())
    }

    // 根据文件名解析并添加密钥
    pub fn add_root_from_filename(&mut self, filename: &str) -> Result<(), crate::Error> {
        let wallet_info =
            crate::utils::file::extract_wallet_address_and_suffix_from_filename(filename)?;

        self.root_info = wallet_info;
        Ok(())
    }

    pub fn deprecate_subkeys(
        &mut self,
        wallet_address: &str,
        subs_path: std::path::PathBuf,
    ) -> Result<(), crate::Error> {
        if self.root_info.address == wallet_address {
            for keystore_info in self.accounts.iter_mut() {
                let old_pk_name = keystore_info.gen_name_with_derivation_path()?;
                let old_path = subs_path.join(old_pk_name);
                keystore_info.suffix = crate::utils::file::Suffix::deprecated_pk();
                let new_pk_name = keystore_info.gen_name_with_derivation_path()?;
                let new_path = subs_path.join(new_pk_name);
                if let Err(e) = std::fs::rename(&old_path, new_path) {
                    tracing::error!("[deprecate_subkeys] Rename {old_path:?} error: {e}");
                };
            }
        }
        Ok(())
    }

    pub fn delete_subkey(
        &mut self,
        wallet_address: &str,
        subs_path: &std::path::PathBuf,
        address: &str,
        chain_code: &ChainCode,
    ) -> Result<(), crate::Error> {
        if self.root_info.address == wallet_address {
            let mut removed_accounts = Vec::new();
            self.accounts.retain(|account| {
                let should_remove = account.address == address && &account.chain_code == chain_code;

                if should_remove {
                    removed_accounts.push(account.clone());
                }

                !should_remove
            });

            for removed_account in removed_accounts {
                let old_pk_name = removed_account.gen_name_with_derivation_path()?;
                let old_path = subs_path.join(old_pk_name);
                if let Err(e) = wallet_utils::file_func::remove_file(&old_path) {
                    tracing::error!("[delete_subkeys] Remove {old_path:?} error: {e}");
                };
            }
        }

        Ok(())
    }

    pub fn recover_subkey(
        &mut self,
        sub_address: &str,
        subs_path: std::path::PathBuf,
    ) -> Result<(), crate::Error> {
        if let Some(keystore_info) = self
            .accounts
            .iter_mut()
            .find(|keystore_info| keystore_info.address == sub_address)
        {
            let old_pk_name = keystore_info.gen_name_with_derivation_path()?;
            let old_path = subs_path.join(old_pk_name);
            keystore_info.suffix = crate::utils::file::Suffix::pk();
            let new_pk_name = keystore_info.gen_name_with_derivation_path()?;
            let new_path = subs_path.join(new_pk_name);
            wallet_utils::file_func::rename(old_path, new_path)?;
        }

        Ok(())
    }

    pub fn get_root(&self) -> RootKeystoreInfo {
        self.root_info.to_owned()
    }

    pub fn get_accounts(&self) -> Vec<SubsKeystoreInfo> {
        self.accounts.to_owned()
    }

    pub fn get_account(&self, address: &str, chain_code: &ChainCode) -> Option<SubsKeystoreInfo> {
        self.accounts
            .iter()
            .find(|a| a.address == address && a.chain_code == *chain_code)
            .map(|info| info.to_owned())
    }

    pub fn get_root_pk_filename(wallet_address: &str) -> Result<String, crate::Error> {
        RootKeystoreInfo::new(crate::utils::file::Suffix::pk(), wallet_address)
            .gen_name_with_address()
    }

    pub fn get_root_seed_filename(wallet_address: &str) -> Result<String, crate::Error> {
        RootKeystoreInfo::new(crate::utils::file::Suffix::seed(), wallet_address)
            .gen_name_with_address()
    }

    pub fn get_root_phrase_filename(wallet_address: &str) -> Result<String, crate::Error> {
        RootKeystoreInfo::new(crate::utils::file::Suffix::phrase(), wallet_address)
            .gen_name_with_address()
    }

    pub fn get_sub_pk_filename(
        address: &str,
        chain_code: &ChainCode,
        raw_derivation_path: &str,
    ) -> Result<String, crate::Error> {
        SubsKeystoreInfo::new(
            raw_derivation_path,
            crate::utils::file::Suffix::pk(),
            chain_code,
            address,
        )
        .gen_name_with_derivation_path()
    }
}
