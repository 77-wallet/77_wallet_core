use std::str::FromStr as _;

use wallet_types::constant::chain_type::*;

#[derive(Debug)]
pub enum HDPath {
    Solana(hdpath::AccountHDPath),
    Other(hdpath::CustomHDPath),
}

impl HDPath {
    pub fn get_account_id(&self) -> Result<u32, crate::Error> {
        Ok((match self {
            HDPath::Solana(path) => path.account(),
            HDPath::Other(path) => match path.0.get(4) {
                Some(hdpath::PathValue::Hardened(index)) => *index | 0x80000000,
                Some(hdpath::PathValue::Normal(index)) => *index,

                _ => {
                    return Err(Into::<crate::Error>::into(hdpath::Error::InvalidStructure));
                }
            },
        }) + 1)
    }

    pub fn get_chain_codes(&self) -> Result<wallet_types::chain::chain::ChainCodes, crate::Error> {
        let coin_type = match self {
            HDPath::Solana(path) => path.coin_type(),
            HDPath::Other(path) => {
                if let Some(hdpath::PathValue::Hardened(coin_type)) = path.0.get(1) {
                    *coin_type
                } else {
                    return Err(Into::<crate::Error>::into(hdpath::Error::InvalidStructure));
                }
            }
        };
        let chain_code = coin_type
            .try_into()
            .map_err(|_| wallet_core::Error::UnknownCoinType(coin_type))?;
        Ok(chain_code)
    }

    pub fn to_derivation_path(&self) -> String {
        match self {
            HDPath::Solana(path) => {
                format!(
                    "m/{}'/{}'/{}'/0",
                    path.purpose().as_value().as_number(),
                    path.coin_type(),
                    path.account()
                )
            }
            HDPath::Other(path) => path.to_string(),
        }
    }
}

pub fn get_account_hd_path_from_path(derivation_path: &str) -> Result<HDPath, crate::Error> {
    let derivation_path = derivation_path.to_uppercase();
    let custom_hd_path = hdpath::CustomHDPath::from_str(&derivation_path).unwrap();

    let coin_type = if let Some(hdpath::PathValue::Hardened(coin_type)) = custom_hd_path.0.get(1) {
        *coin_type
    } else {
        return Err(Into::<crate::Error>::into(hdpath::Error::InvalidStructure));
    };

    let hd_path = match coin_type {
        ETH_TYPE | TRON_TYPE | BTC_TYPE | BTC_86_TYPE | LTC_TYPE | DOG_TYPE => {
            HDPath::Other(custom_hd_path)
        }
        SOLANA_TYPE => HDPath::Solana(
            hdpath::AccountHDPath::from_str(&derivation_path)
                .map_err(|e| Into::<crate::Error>::into(e))?,
        ),
        _ => return Err(wallet_core::Error::UnknownChainCode.into()),
    };
    Ok(hd_path)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use hdpath::AccountHDPath;
    use wallet_types::{
        chain::chain::{ChainCode, ChainCodes},
        constant::chain_type::SOLANA_TYPE,
    };
    use wallet_utils::init_test_log;

    use crate::derivation_path::{HDPath, get_account_hd_path_from_path};

    // use coins_bip32::path::DerivationPath;

    #[test]
    fn test_parse_solana_path() {
        init_test_log();
        let derivation_path = "m/44'/501'/0'/0'";
        let result = get_account_hd_path_from_path(derivation_path);
        assert!(result.is_ok());

        let hd_path = result.unwrap();
        match hd_path {
            HDPath::Solana(path) => {
                assert_eq!(path.coin_type(), SOLANA_TYPE);
                assert_eq!(path.account(), 0);
            }
            _ => panic!("Expected Solana HDPath"),
        }
    }

    #[test]
    fn test_parse_eth_path() {
        init_test_log();
        let derivation_path = "m/44'/60'/0'/0/2147478971'";
        let result = get_account_hd_path_from_path(derivation_path);
        println!("result: {result:?}");
        assert!(result.is_ok());

        let hd_path = result.unwrap();
        let chain_code = hd_path.get_chain_codes().unwrap();
        assert_eq!(
            chain_code,
            ChainCodes(vec![ChainCode::Ethereum, ChainCode::BnbSmartChain])
        );
        let account_id = hd_path.get_account_id().unwrap();
        assert_eq!(account_id, 2147478972);

        // match hd_path {
        //     HDPath::Other(path) => {
        //         assert_eq!(path.coin_type(), ETH_TYPE);
        //         assert_eq!(path.index(), 2147478971);
        //     }
        //     _ => panic!("Expected Other HDPath for Ethereum"),
        // }
    }

    #[test]
    fn test_get_account_id_solana() {
        let derivation_path = "m/44'/501'/0'/0'";
        let hd_path = get_account_hd_path_from_path(derivation_path).unwrap();
        println!("hd_path: {hd_path:?}");

        let account_id = hd_path.get_account_id().unwrap();
        assert_eq!(account_id, 1);
    }

    #[test]
    fn test_get_account_id_eth() {
        let derivation_path = "m/44'/60'/0'/0/0";
        let hd_path = get_account_hd_path_from_path(derivation_path).unwrap();

        let account_id = hd_path.get_account_id().unwrap();
        assert_eq!(account_id, 1);
    }

    #[test]
    fn test_invalid_path() {
        let derivation_path = "m/44'/999'/0'/0/0"; // Invalid coin type
        let result = get_account_hd_path_from_path(derivation_path);
        assert!(result.is_err());
    }

    #[test]
    fn parse_derivation_path() {
        let derivation_path = "m/44'/60'/0'/0/0";
        // let path = DerivationPath::from_str(derivation_path).unwrap();
        let hd_path = AccountHDPath::from_str(derivation_path).unwrap();
        // prints "m/44'/0'/0'/0/0"
        println!("{:?}", hd_path);

        // prints "0", which is account id
        println!("{:?}", hd_path.account());

        // prints: "purpose: Pubkey, coin: 0, account: 0, change: 0, index: 0"
        println!(
            "purpose: {:?}, coin: {}, account: {}",
            hd_path.purpose(),
            hd_path.coin_type(),
            hd_path.account(),
        )
        // println!(
        //     "purpose: {:?}, coin: {}, account: {}, change: {}, index: {}",
        //     hd_path.purpose(),
        //     hd_path.coin_type(),
        //     hd_path.account(),
        //     hd_path.change(),
        //     hd_path.index()
        // );
    }
}
