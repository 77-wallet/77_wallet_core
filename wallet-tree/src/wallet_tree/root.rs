use serde::Serialize;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct RootKeystoreInfo {
    pub address: String,
    pub suffix: crate::utils::file::Suffix,
}

impl RootKeystoreInfo {
    pub fn new(suffix: crate::utils::file::Suffix, address: &str) -> Self {
        Self {
            address: address.to_string(),
            suffix,
        }
    }

    pub fn gen_name_with_address(&self) -> Result<String, crate::Error> {
        let name = format!("{}-{}", self.address, self.suffix.gen_suffix());
        Ok(name)
    }

    pub fn gen_name_with_derivation_path(
        &self,
        raw_derivation_path: &str,
    ) -> Result<String, crate::Error> {
        let percent_derivation_path =
            wallet_utils::parse_func::derivation_path_percent_decode(raw_derivation_path);
        let derivation_path =
            wallet_utils::parse_func::decode_from_percent(percent_derivation_path)?.to_string();

        let name = format!(
            "{}-{}-{}",
            self.address,
            derivation_path,
            self.suffix.gen_suffix()
        );
        Ok(name)
    }
}
