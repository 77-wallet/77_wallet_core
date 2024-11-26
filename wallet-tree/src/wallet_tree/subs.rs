use serde::Serialize;
use wallet_types::chain::chain::ChainCode;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct SubsKeystoreInfo {
    pub derivation_path: String,
    pub address: String,
    pub chain_code: ChainCode,
    pub suffix: crate::utils::file::Suffix,
}

impl SubsKeystoreInfo {
    pub fn new(
        derivation_path: &str,
        suffix: crate::utils::file::Suffix,
        chain_code: &ChainCode,
        address: &str,
    ) -> Self {
        Self {
            derivation_path: derivation_path.to_string(),
            address: address.to_string(),
            chain_code: chain_code.clone(),
            suffix,
        }
    }

    pub fn gen_name_with_address(&self) -> String {
        let name = format!(
            "{}-{}-{}",
            self.chain_code,
            self.address,
            self.suffix.gen_suffix()
        );
        name
    }

    pub fn gen_name_with_derivation_path(&self) -> Result<String, crate::Error> {
        let derivation_path =
            wallet_utils::parse_func::derivation_path_percent_encode(&self.derivation_path);

        let name = format!(
            "{}-{}-{}-{}",
            self.chain_code,
            self.address,
            derivation_path,
            self.suffix.gen_suffix()
        );
        Ok(name)
    }
}
