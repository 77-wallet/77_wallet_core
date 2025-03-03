use wallet_types::chain::chain::ChainCode;

use super::{WalletTree, WalletTreeOps};

// pub struct LegacyAdapter {
//     naming: LegacyNaming,
//     layout: LegacyLayout,
// }

impl WalletTreeOps for WalletTree {
    type Tree = WalletTree;
    fn traverse(wallet_dir: &std::path::PathBuf) -> Result<Self::Tree, crate::Error>
    where
        Self: Sized,
    {
        WalletTree::traverse_directory_structure(wallet_dir)
    }

    fn delete_subkey(
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

    fn deprecate_subkeys(
        mut self,
        wallet_address: &str,
        subs_path: std::path::PathBuf,
    ) -> Result<(), crate::Error> {
        let wallet = self.get_mut_wallet_branch(&wallet_address.to_string())?;

        wallet.deprecate_subkeys(&wallet_address.to_string(), subs_path)?;
        Ok(())
    }

    fn get_root_info(
        &self,
        wallet_address: &str,
    ) -> Result<super::root::RootKeystoreInfo, crate::Error> {
        let wallet = self.get_wallet_branch(wallet_address)?;
        Ok(wallet.root_info.clone())
    }
    // 其他方法适配...
}
