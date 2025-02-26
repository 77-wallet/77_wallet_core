use std::path::Path;

use crate::wallet::{
    phrase::builder::{PhraseDecryptorBuilder, PhraseEncryptorBuilder},
    prikey::builder::{PrikeyDecryptorBuilder, PrikeyEncryptorBuilder},
    seed::builder::{SeedDecryptorBuilder, SeedEncryptorBuilder},
};

#[derive(Debug, Clone, Default)]
pub struct Keystore {}

impl Keystore {
    pub fn store_root_private_key<P: AsRef<Path>>(
        // address: &str,
        name: &str,
        private_key: &[u8],
        file_path: &P,
        password: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<crate::wallet::prikey::PkWallet, crate::Error> {
        let mut rng = rand::thread_rng();
        // let name = RootKeystoreInfo::new(crate::utils::file::Suffix::pk(), address)
        //     .gen_name_with_address()?;
        let address_generator = wallet_chain_instance::instance::eth::address::EthGenAddress::new(
            wallet_types::chain::chain::ChainCode::Ethereum,
        );

        use crate::wallet::WalletEncrypt as _;
        let (pk_wallet, _) = PrikeyEncryptorBuilder::new(
            file_path.as_ref(),
            &mut rng,
            private_key,
            password,
            Some(name),
            Box::new(address_generator),
            algorithm,
        )
        .encrypt_keystore()?;

        Ok(pk_wallet)
    }

    pub fn store_seed_keystore<P: AsRef<Path>>(
        // address: &str,
        name: &str,
        seed: &[u8],
        directory: &P,
        password: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<crate::wallet::seed::SeedWallet, crate::Error> {
        let mut rng = rand::thread_rng();
        // let name = RootKeystoreInfo::new(crate::utils::file::Suffix::seed(), address)
        //     .gen_name_with_address()?;

        use crate::wallet::WalletEncrypt as _;
        let (seed_wallet, _) = SeedEncryptorBuilder::new(
            directory.as_ref(),
            &mut rng,
            seed,
            password,
            Some(name),
            algorithm,
        )
        .encrypt_keystore()?;
        Ok(seed_wallet)
    }

    pub fn store_phrase_keystore<P: AsRef<Path>>(
        // address: &str,
        name: &str,
        phrase: &str,
        directory: &P,
        password: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<crate::wallet::phrase::PhraseWallet, crate::Error> {
        let mut rng = rand::thread_rng();
        // let name = RootKeystoreInfo::new(crate::utils::file::Suffix::phrase(), address)
        //     .gen_name_with_address()?;
        let phrase_vec = wallet_utils::conversion::str_to_vec(phrase);

        use crate::wallet::WalletEncrypt as _;
        let (phrase_wallet, _) = PhraseEncryptorBuilder::new(
            directory.as_ref(),
            &mut rng,
            phrase_vec,
            password,
            Some(name),
            algorithm,
        )
        .encrypt_keystore()?;
        Ok(phrase_wallet)
    }

    pub fn store_sub_private_key<P: AsRef<Path>>(
        address_generator: Box<
            dyn wallet_core::address::GenAddress<
                Address = wallet_chain_instance::instance::Address,
                Error = wallet_chain_instance::Error,
            >,
        >,
        private_key: Vec<u8>,
        file_path: P,
        password: &str,
        address: &str,
        derivation_path: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<crate::wallet::prikey::PkWallet, crate::Error> {
        let mut rng = rand::thread_rng();

        let name = wallet_tree::wallet_tree::subs::SubsKeystoreInfo::new(
            derivation_path,
            wallet_tree::utils::file::Suffix::pk(),
            address_generator.chain_code(),
            address,
        )
        .gen_name_with_derivation_path()?;

        use crate::wallet::WalletEncrypt as _;
        let (pk_wallet, _) = PrikeyEncryptorBuilder::new(
            file_path.as_ref(),
            &mut rng,
            private_key,
            password,
            Some(&name),
            address_generator,
            algorithm,
        )
        .encrypt_keystore()?;

        Ok(pk_wallet)
    }

    pub fn load_private_key_keystore<P: AsRef<Path>>(
        file_path: P,
        password: &str,
        address_generator: Box<
            dyn wallet_core::address::GenAddress<
                Address = wallet_chain_instance::instance::Address,
                Error = wallet_chain_instance::Error,
            >,
        >,
    ) -> Result<crate::wallet::prikey::PkWallet, crate::Error> {
        use crate::wallet::WalletDecrypt as _;
        let prikey_wallet =
            PrikeyDecryptorBuilder::new(file_path.as_ref(), password, address_generator)
                .decrypt_keystore()?;

        Ok(prikey_wallet)
    }

    pub fn load_phrase_keystore<P: AsRef<Path>>(
        address: &str,
        directory: &P,
        password: &str,
    ) -> Result<crate::wallet::phrase::PhraseWallet, crate::Error> {
        let name = wallet_tree::wallet_tree::root::RootKeystoreInfo::new(
            wallet_tree::utils::file::Suffix::phrase(),
            address,
        )
        .gen_name_with_address()?;
        let path = directory.as_ref().join(name);

        use crate::wallet::WalletDecrypt as _;
        let phrase_wallet = PhraseDecryptorBuilder::new(path, password).decrypt_keystore()?;

        Ok(phrase_wallet)
    }

    pub fn load_seed_keystore<P: AsRef<Path>>(
        address: &str,
        directory: P,
        password: &str,
    ) -> Result<crate::wallet::seed::SeedWallet, crate::Error> {
        let name = wallet_tree::wallet_tree::root::RootKeystoreInfo::new(
            wallet_tree::utils::file::Suffix::seed(),
            address,
        )
        .gen_name_with_address()?;
        let path = directory.as_ref().join(name);

        use crate::wallet::WalletDecrypt as _;
        let recovered_wallet = SeedDecryptorBuilder::new(path, password).decrypt_keystore()?;
        Ok(recovered_wallet)
    }
}
