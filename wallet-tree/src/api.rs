use wallet_keystore::{
    wallet::{phrase::PhraseWallet, prikey::PkWallet, seed::SeedWallet},
    KdfAlgorithm, KeystoreJson,
};

use crate::{naming::FileType, wallet_tree::WalletTreeOps};

pub struct KeystoreApi;

impl KeystoreApi {
    // 传入助记词、盐，生成密钥，创建根Keystore，并且保存到文件
    pub fn initialize_root_keystore(
        wallet_tree: Box<dyn WalletTreeOps>,
        address: &str,
        private_key: &[u8],
        seed: &[u8],
        phrase: &str,
        path: &std::path::PathBuf,
        password: &str,
        algorithm: KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        // let root = wallet_tree.get_wallet_branch(address)?.get_root();
        let naming = wallet_tree.naming();

        let phrase_meta = naming.generate_filemeta(FileType::Phrase, &address, None, None, None)?;
        let pk_meta = naming.generate_filemeta(FileType::PrivateKey, &address, None, None, None)?;
        let seed_meta = naming.generate_filemeta(FileType::Seed, &address, None, None, None)?;

        // WalletBranch::get_root_pk_filename(address)?;
        let pk_filename = wallet_tree.naming().encode(pk_meta)?;
        let seed_filename = wallet_tree.naming().encode(seed_meta)?;
        let phrase_filename = wallet_tree.naming().encode(phrase_meta)?;
        crate::Keystore::store_data(
            &pk_filename,
            private_key,
            &path,
            password,
            algorithm.clone(),
        )?;
        // let name = WalletBranch::get_root_seed_filename(address)?;

        crate::Keystore::store_data(&seed_filename, seed, &path, password, algorithm.clone())?;
        // let name = WalletBranch::get_root_phrase_filename(address)?;
        crate::Keystore::store_data(&phrase_filename, phrase, &path, password, algorithm)?;
        Ok(())
    }

    // 传入derivation_path，由根私钥派生出子私钥，创建子Keystore，并生成keystore文件
    pub fn initialize_child_keystores<P: AsRef<std::path::Path>>(
        wallet_tree: Box<dyn WalletTreeOps>,
        subkeys: Vec<crate::io::BulkSubkey>,
        subs_path: P,
        password: &str,
        algorithm: KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let naming = wallet_tree.naming();

        wallet_tree
            .io()
            .store_subkeys_bulk(naming, subkeys, &subs_path, password, algorithm)?;

        Ok(())
    }
    // pub fn initialize_child_keystore<P: AsRef<std::path::Path>>(
    //     wallet_tree: Box<dyn WalletTreeOps>,
    //     account_index_map: &wallet_utils::address::AccountIndexMap,
    //     instance: &wallet_chain_instance::instance::ChainObject,
    //     seed: &[u8],
    //     derivation_path: &str,
    //     subs_path: P,
    //     password: &str,
    //     algorithm: KdfAlgorithm,
    // ) -> Result<(), crate::Error> {
    //     let naming = wallet_tree.naming();

    //     let gen_address = instance.gen_gen_address()?;
    //     let keypair = instance.gen_keypair_with_derivation_path(seed, derivation_path)?;
    //     let address = keypair.address();
    //     let private_key = keypair.private_key_bytes()?;

    //     wallet_tree.io().store_subkey(
    //         naming,
    //         account_index_map,
    //         &address,
    //         &gen_address.chain_code().to_string(),
    //         derivation_path,
    //         &private_key,
    //         &subs_path,
    //         password,
    //         algorithm,
    //     )?;

    //     Ok(())
    // }

    pub fn get_private_key<P: AsRef<std::path::Path> + std::fmt::Debug>(
        password: &str,
        path: P,
    ) -> Result<Vec<u8>, crate::Error> {
        let pkwallet = crate::Keystore::load_data::<_, PkWallet>(path, password)?;
        Ok(pkwallet.pkey().to_vec())
    }

    pub fn check_wallet_address(
        language_code: u8,
        phrase: &str,
        salt: &str,
        address: wallet_chain_instance::instance::Address,
    ) -> Result<(), crate::Error> {
        use wallet_core::address::GenAddress as _;
        let (master_key, _) = wallet_core::xpriv::generate_master_key(language_code, phrase, salt)?;
        let signingkey: &coins_bip32::ecdsa::SigningKey = master_key.as_ref();
        let pkey = signingkey.to_bytes();

        let data = Box::new(
            wallet_chain_instance::instance::eth::address::EthGenAddress::new(
                wallet_types::chain::chain::ChainCode::Ethereum,
            ),
        );
        let generated_address = data.generate(&pkey)?;

        if generated_address.ne(&address) {
            return Err(crate::Error::Parase);
        }
        Ok(())
    }

    pub fn load_sub_pk(
        wallet_tree: &Box<dyn WalletTreeOps>,
        account_index_map: Option<&wallet_utils::address::AccountIndexMap>,
        subs_dir: &std::path::PathBuf,
        address: &str,
        chain_code: &str,
        derivation_path: &str,
        password: &str,
    ) -> Result<Vec<u8>, crate::Error> {
        // let recovered_wallet =
        //     crate::Keystore::load_seed_keystore(wallet_address, root_dir, password)?;
        let naming = wallet_tree.naming();

        // let pk_meta = naming.generate_filemeta(
        //     FileType::DerivedData,
        //     &address,
        //     account_index_map,
        //     Some(chain_code.to_string()),
        //     Some(derivation_path.to_string()),
        // )?;
        // let pk_filename = naming.encode(pk_meta)?;

        let pk = wallet_tree.io().load(
            naming,
            account_index_map,
            address,
            chain_code,
            derivation_path,
            subs_dir,
            password,
        )?;
        // let pk = crate::Keystore::load_data::<_, PkWallet>(subs_dir.join(pk_filename), password)?;
        Ok(pk)
    }

    pub fn load_seed(
        wallet_tree: &Box<dyn WalletTreeOps>,
        root_dir: &std::path::PathBuf,
        wallet_address: &str,
        password: &str,
    ) -> Result<Vec<u8>, crate::Error> {
        // let recovered_wallet =
        //     crate::Keystore::load_seed_keystore(wallet_address, root_dir, password)?;
        let naming = wallet_tree.naming();

        let seed_meta =
            naming.generate_filemeta(FileType::Seed, &wallet_address, None, None, None)?;
        let seed_filename = naming.encode(seed_meta)?;

        let seed =
            crate::Keystore::load_data::<_, SeedWallet>(root_dir.join(seed_filename), password)?;
        Ok(seed.into_seed())
    }

    pub fn load_phrase(
        wallet_tree: &Box<dyn WalletTreeOps>,
        root_dir: &std::path::PathBuf,
        wallet_address: &str,
        password: &str,
    ) -> Result<String, crate::Error> {
        // let recovered_wallet =
        //     crate::Keystore::load_seed_keystore(wallet_address, root_dir, password)?;
        let naming = wallet_tree.naming();

        let phrase_meta =
            naming.generate_filemeta(FileType::Phrase, &wallet_address, None, None, None)?;
        let phrase_filename = naming.encode(phrase_meta)?;

        let phrase_wallet = crate::Keystore::load_data::<_, PhraseWallet>(
            root_dir.join(phrase_filename),
            password,
        )?;
        Ok(phrase_wallet.phrase)
    }

    pub fn update_root_password(
        root_dir: std::path::PathBuf,
        wallet_tree: Box<dyn WalletTreeOps>,
        wallet_address: &str,
        old_password: &str,
        new_password: &str,
        algorithm: KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let naming = wallet_tree.naming();

        let pk_meta =
            naming.generate_filemeta(FileType::PrivateKey, &wallet_address, None, None, None)?;
        let pk_filename = naming.encode(pk_meta)?;

        let path = root_dir.join(pk_filename);
        let private_key = Self::get_private_key(old_password, &path)?;

        // let seed_meta = naming.generate_filemeta(FileType::Seed, &wallet_address, None, None)?;
        // let seed_filename = naming.encode(seed_meta)?;

        // let seed = crate::Keystore::load_data::<_, SeedWallet>(
        //     root_dir.join(seed_filename),
        //     old_password,
        // )?;

        let seed = Self::load_seed(&wallet_tree, &root_dir, wallet_address, old_password)?;
        // let seed = crate::Keystore::load_seed_keystore(wallet_address, &root_dir, old_password)?
        //     .into_seed();

        // let phrase_meta =
        //     naming.generate_filemeta(FileType::Phrase, &wallet_address, None, None)?;
        // let phrase_filename = naming.encode(phrase_meta)?;

        // let phrase_wallet = crate::Keystore::load_data::<_, PhraseWallet>(
        //     root_dir.join(phrase_filename),
        //     old_password,
        // )?;

        let phrase = Self::load_phrase(&wallet_tree, &root_dir, wallet_address, old_password)?;
        // let phrase_wallet =
        //     crate::Keystore::load_phrase_keystore(wallet_address, &root_dir, old_password)?;

        // let phrase_wallet = crate::Keystore::load_data(path, password)
        Self::initialize_root_keystore(
            wallet_tree,
            wallet_address,
            &private_key,
            &seed,
            &phrase,
            &root_dir,
            new_password,
            algorithm,
        )
    }

    pub fn update_child_password(
        subs_dir: std::path::PathBuf,
        instance: wallet_chain_instance::instance::ChainObject,
        wallet_tree: Box<dyn WalletTreeOps>,
        wallet_address: &str,
        address: &str,
        old_password: &str,
        new_password: &str,
        algorithm: KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        // let gen_address = instance.gen_gen_address()?;

        if let Some(account) = wallet_tree
            .get_wallet_branch(wallet_address)?
            .get_account(address, instance.chain_code())
        {
            let meta = account.get_filemeta();
            let filename = wallet_tree.naming().encode(meta)?;
            // let filename = account.gen_name_with_derivation_path()?;
            let path = subs_dir.join(&filename);
            let pk = crate::api::KeystoreApi::get_private_key(old_password, &path)?;

            // let name = wallet_tree::wallet_tree::subs::SubsKeystoreInfo::new(
            //     &account.derivation_path,
            //     wallet_tree::utils::file::Suffix::pk(),
            //     gen_address.chain_code(),
            //     &address,
            // )
            // .gen_name_with_derivation_path()?;

            crate::Keystore::store_data(&filename, pk, &subs_dir, new_password, algorithm)?;
        }

        Ok(())
    }

    pub fn generate_master_key_info(
        language_code: u8,
        phrase: &str,
        salt: &str,
    ) -> Result<RootInfo, crate::Error> {
        let (master_key, seed) =
            wallet_core::xpriv::generate_master_key(language_code, phrase, salt)?;
        let signingkey: &coins_bip32::ecdsa::SigningKey = master_key.as_ref();
        let private_key = signingkey.to_bytes();
        let address = alloy::signers::utils::secret_key_to_address(signingkey);
        Ok(RootInfo {
            private_key: private_key.to_vec(),
            phrase: phrase.to_string(),
            seed,
            address,
        })
    }

    pub fn reset_and_store_root_keys(
        wallet_tree: Box<dyn WalletTreeOps>,
        storage_path: &std::path::PathBuf,
        root_info: RootInfo,
        password: &str,
        algorithm: KdfAlgorithm,
    ) -> Result<String, crate::Error> {
        // 清理并重新创建目录
        wallet_utils::file_func::recreate_dir_all(storage_path)?;

        Self::initialize_root_keystore(
            wallet_tree,
            &root_info.address.to_string(),
            &root_info.private_key,
            &root_info.seed,
            &root_info.phrase,
            storage_path,
            password,
            algorithm,
        )?;

        Ok(root_info.address.to_string())
    }

    pub fn read_keystore<P: AsRef<std::path::Path> + std::fmt::Debug>(
        path: P,
    ) -> Result<KeystoreJson, crate::Error> {
        let mut contents = String::new();
        wallet_utils::file_func::read(&mut contents, path)?;
        Ok(wallet_utils::serde_func::serde_from_str(&contents)?)
    }
}

pub struct RootInfo {
    pub private_key: Vec<u8>,
    pub phrase: String,
    pub seed: Vec<u8>,
    pub address: alloy::primitives::Address,
}
