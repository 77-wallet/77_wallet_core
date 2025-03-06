use std::fs;

use serde::Serialize;
use wallet_keystore::KeystoreBuilder;

use crate::naming::{
    modern::{DerivedMetadata, KeyMeta, KeyMetas, KeystoreData},
    FileType, NamingStrategy,
};

use super::IoStrategy;

#[derive(Debug, Default, PartialEq, Clone, Serialize)]
pub struct ModernIo;

impl IoStrategy for ModernIo {
    fn store(
        &self,
        name: &str,
        data: &dyn AsRef<[u8]>,
        file_path: &dyn AsRef<std::path::Path>,
        password: &str,
        algorithm: wallet_keystore::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let rng = rand::thread_rng();
        KeystoreBuilder::new_encrypt(file_path, password, data, rng, algorithm, &name).save()?;
        Ok(())
    }

    fn load(
        &self,
        naming: Box<dyn crate::naming::NamingStrategy>,
        account_index_map: Option<&wallet_utils::address::AccountIndexMap>,
        address: &str,
        chain_code: &str,
        derivation_path: &str,
        subs_dir: &dyn AsRef<std::path::Path>,
        password: &str,
        // ) -> Result<Box<dyn TryFrom<RecoverableData>>, crate::Error>;
    ) -> Result<Vec<u8>, crate::Error> {
        let pk_meta = naming.generate_filemeta(
            FileType::DerivedData,
            &address,
            account_index_map,
            Some(chain_code.to_string()),
            Some(derivation_path.to_string()),
        )?;
        let pk_filename = naming.encode(pk_meta)?;

        let data =
            KeystoreBuilder::new_decrypt(subs_dir.as_ref().join(pk_filename), password).load()?;

        let derived_data: KeystoreData = data.try_into()?;
        tracing::info!("derived_data: {:#?}", derived_data);

        for (k, v) in derived_data.iter() {
            if let Ok(meta) = KeyMeta::decode(k)
                && meta.address == address
                && meta.chain_code == chain_code
                && meta.derivation_path == derivation_path
            {
                return Ok(v.to_vec());
            }
        }

        return Err(crate::Error::PrivateKeyNotFound);
    }

    // fn load_subkey(){}

    fn store_subkey(
        &self,
        naming: Box<dyn NamingStrategy>,
        account_index_map: &wallet_utils::address::AccountIndexMap,
        address: &str,
        chain_code: &str,
        derivation_path: &str,
        data: &dyn AsRef<[u8]>,
        file_path: &dyn AsRef<std::path::Path>,
        password: &str,
        algorithm: wallet_keystore::KdfAlgorithm,
    ) -> Result<(), crate::Error> {
        let account_idx = account_index_map.account_id;
        let base_path = file_path.as_ref();
        // let data_path = base_path.join("subs/derived_keys.keystore");
        let meta_path = base_path.join("derived_meta.json");

        tracing::warn!("store_subkey ============ 1");
        // 1. 处理元数据
        let mut metadata = if meta_path.exists() {
            let content = fs::read_to_string(&meta_path).unwrap();
            serde_json::from_str(&content).unwrap_or_else(|_| DerivedMetadata::default())
        } else {
            DerivedMetadata::default()
        };
        tracing::warn!("store_subkey ============ 2");

        let meta = naming.generate_filemeta(
            FileType::DerivedData,
            &address,
            Some(account_index_map),
            Some(chain_code.to_string()),
            Some(derivation_path.to_string()),
        )?;

        tracing::warn!("store_subkey ============ 3");
        // 生成密钥文件名
        let key_filename = naming.encode(meta)?;
        // let key_filename = format!("key{}.keystore", account_idx);

        tracing::warn!("store_subkey ============ 4");
        // 添加新条目
        metadata
            .accounts
            .entry(account_idx)
            .or_insert(KeyMetas::default())
            .push(KeyMeta {
                chain_code: chain_code.to_string(),
                address: address.to_string(),
                derivation_path: derivation_path.to_string(),
            });

        tracing::warn!("store_subkey ============ 5");
        // 写入元数据
        let contents = wallet_utils::serde_func::serde_to_string(&metadata)?;
        tracing::info!("meta_path: {meta_path:?}");
        wallet_utils::file_func::write_all(&meta_path, contents.as_bytes())?;

        // 2. 处理密钥数据

        // let data = KeystoreBuilder::new_decrypt(data_path, password).load()?;

        tracing::warn!("store_subkey ============ 6");
        // let data_path = base_path;

        let data_path = base_path.join(&key_filename);
        let mut derived_data = crate::naming::modern::KeystoreData::default();
        if data_path.exists() {
            let keystore = KeystoreBuilder::new_decrypt(&data_path, password).load()?;
            derived_data = keystore.try_into()?;
            // let pkwallet =
            // existing_data = keystore.data;
        }

        let key = KeyMeta {
            chain_code: chain_code.to_string(),
            address: address.to_string(),
            derivation_path: derivation_path.to_string(),
        };

        derived_data.insert(key.encode(), data.as_ref().to_vec());

        let val = wallet_utils::serde_func::serde_to_string(&derived_data)?;
        tracing::info!("val: {val:?}");
        let rng = rand::thread_rng();
        KeystoreBuilder::new_encrypt(
            &base_path,
            password,
            val.as_bytes(), // 原始二进制数据
            rng,
            algorithm,
            &key_filename, // 唯一标识
        )
        .save()?;
        tracing::warn!("store_subkey ============ 7");
        Ok(())
    }
}
