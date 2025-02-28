use wallet_tree::wallet_tree::WalletBranch;

pub struct KeystoreApi;

impl KeystoreApi {
    // 传入助记词、盐，生成密钥，创建根Keystore，并且保存到文件
    pub fn initialize_root_keystore(
        address: &str,
        private_key: &[u8],
        seed: &[u8],
        phrase: &str,
        path: &std::path::PathBuf,
        password: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let name = WalletBranch::get_root_pk_filename(address)?;

        crate::Keystore::store_root_private_key(
            &name,
            private_key,
            &path,
            password,
            algorithm.clone(),
        )?;
        let name = WalletBranch::get_root_seed_filename(address)?;

        crate::Keystore::store_seed_keystore(&name, seed, &path, password, algorithm.clone())?;
        let name = WalletBranch::get_root_phrase_filename(address)?;
        crate::Keystore::store_phrase_keystore(&name, phrase, &path, password, algorithm)?;
        Ok(())
    }

    // 传入derivation_path，由根私钥派生出子私钥，创建子Keystore，并生成keystore文件
    pub fn initialize_child_keystore<P: AsRef<std::path::Path>>(
        instance: &wallet_chain_instance::instance::ChainObject,
        seed: &[u8],
        derivation_path: &str,
        path: P,
        password: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let gen_address = instance.gen_gen_address()?;
        let keypair = instance.gen_keypair_with_derivation_path(seed, derivation_path)?;

        let address = keypair.address();
        let private_key = keypair.private_key_bytes()?;
        crate::Keystore::store_sub_private_key(
            gen_address,
            private_key,
            path,
            password,
            &address,
            derivation_path,
            algorithm,
        )?;

        Ok(())
    }

    pub fn get_private_key<P: AsRef<std::path::Path> + std::fmt::Debug>(
        password: &str,
        path: P,
    ) -> Result<Vec<u8>, crate::Error> {
        let recovered_wallet = crate::Keystore::load_private_key_keystore(path, password)?;
        Ok(recovered_wallet.pkey().to_vec())
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

        // let wallet = crate::wallet::prikey::PkWallet::from_pkey(
        //     &pkey,
        //     // Box::new(
        //     //     wallet_chain_instance::instance::eth::address::EthGenAddress::new(
        //     //         wallet_types::chain::chain::ChainCode::Ethereum,
        //     //     ),
        //     // ),
        // )?;
        // let generated_address = wallet.address();
        if generated_address.ne(&address) {
            return Err(crate::Error::Parase);
        }
        Ok(())
    }

    pub fn load_seed(
        root_dir: std::path::PathBuf,
        wallet_address: &str,
        password: &str,
    ) -> Result<Vec<u8>, crate::Error> {
        let recovered_wallet =
            crate::Keystore::load_seed_keystore(wallet_address, root_dir, password)?;
        Ok(recovered_wallet.into_seed())
    }

    pub fn update_root_password(
        root_dir: std::path::PathBuf,
        wallet_tree: wallet_tree::wallet_tree::WalletTree,
        wallet_address: &str,
        old_password: &str,
        new_password: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let path = root_dir.join(
            wallet_tree
                .get_wallet_branch(wallet_address)?
                .get_root()
                .gen_name_with_address()?,
        );

        let private_key = Self::get_private_key(old_password, &path)?;
        let seed = crate::Keystore::load_seed_keystore(wallet_address, &root_dir, old_password)?
            .into_seed();
        let phrase_wallet =
            crate::Keystore::load_phrase_keystore(wallet_address, &root_dir, old_password)?;
        Self::initialize_root_keystore(
            wallet_address,
            &private_key,
            &seed,
            phrase_wallet.phrase(),
            &root_dir,
            new_password,
            algorithm,
        )
    }

    pub fn update_child_password(
        subs_dir: std::path::PathBuf,
        instance: wallet_chain_instance::instance::ChainObject,
        wallet_tree: wallet_tree::wallet_tree::WalletTree,
        wallet_address: &str,
        address: &str,
        old_password: &str,
        new_password: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let gen_address = instance.gen_gen_address()?;
        if let Some(account) = wallet_tree
            .get_wallet_branch(wallet_address)?
            .get_account(address, instance.chain_code())
        {
            let filename = account.gen_name_with_derivation_path()?;
            let path = subs_dir.join(filename);
            let pk = crate::api::KeystoreApi::get_private_key(old_password, &path)?;

            crate::Keystore::store_sub_private_key(
                gen_address,
                pk,
                subs_dir,
                new_password,
                address,
                &account.derivation_path,
                algorithm,
            )?;
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
        storage_path: &std::path::PathBuf,
        root_info: RootInfo,
        password: &str,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Result<String, crate::Error> {
        // 清理并重新创建目录
        wallet_utils::file_func::recreate_dir_all(storage_path)?;

        Self::initialize_root_keystore(
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
    ) -> Result<crate::keystore::json::KeystoreJson, crate::Error> {
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
