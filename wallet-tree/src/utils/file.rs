use crate::wallet_tree::{root::RootKeystoreInfo, subs::SubsKeystoreInfo};

pub fn extract_wallet_address_and_suffix_from_filename(
    filename: &str,
) -> Result<RootKeystoreInfo, crate::Error> {
    let parts: Vec<&str> = filename.split('-').collect();
    if parts.len() >= 2 {
        let address = parts[0].to_string();
        let suffix = parts[1];
        let deprecated = suffix.starts_with("deprecated");
        let suffix = if suffix.ends_with("pk") {
            if deprecated {
                Suffix::deprecated_pk()
            } else {
                Suffix::pk()
            }
        } else if suffix.ends_with("seed") {
            Suffix::seed()
        } else {
            return Err(crate::Error::FilenameInvalid);
        };

        Ok(RootKeystoreInfo { address, suffix })
    } else {
        Err(crate::Error::FilenameInvalid)
    }
}

pub fn extract_sub_address_and_derive_path_from_filename(
    filename: &str,
) -> Result<SubsKeystoreInfo, crate::Error> {
    let parts: Vec<&str> = filename.split('-').collect();
    if parts.len() >= 4 {
        let chain_code = parts[0];
        let address = parts[1].to_string();
        let encoded_derivation_path = parts[2].to_string();
        let suffix = parts[3];

        let deprecated = suffix.starts_with("deprecated");
        let suffix = if suffix.ends_with("pk") {
            if deprecated {
                Suffix::deprecated_pk()
            } else {
                Suffix::pk()
            }
        } else {
            return Err(crate::Error::FilenameInvalid);
        };
        let derivation_path =
            wallet_utils::parse_func::derivation_path_percent_decode(&encoded_derivation_path)?
                .to_string();
        // let derivation_path =
        // wallet_utils::parse_func::decode_from_percent(percent_derivation_path)?.to_string();
        Ok(SubsKeystoreInfo {
            derivation_path,
            address,
            chain_code: chain_code.try_into()?,
            suffix,
        })
    } else {
        Err(crate::Error::FilenameInvalid)
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum Suffix {
    Pk { deprecated: bool },
    Seed,
    Phrase,
}

impl Suffix {
    pub fn pk() -> Suffix {
        Suffix::Pk { deprecated: false }
    }

    pub fn deprecated_pk() -> Suffix {
        Suffix::Pk { deprecated: true }
    }

    pub fn seed() -> Suffix {
        Suffix::Seed
    }

    pub fn phrase() -> Suffix {
        Suffix::Phrase
    }

    pub fn gen_suffix(&self) -> String {
        match self {
            Suffix::Pk { deprecated } => {
                if *deprecated {
                    "deprecated_pk".to_string()
                } else {
                    "pk".to_string()
                }
            }
            Suffix::Seed => "seed".to_string(),
            Suffix::Phrase => "phrase".to_string(),
        }
    }
}
