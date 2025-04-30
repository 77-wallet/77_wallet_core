#[derive(Debug, PartialEq, Clone, serde::Serialize, Copy)]
pub enum ChainCode {
    Tron,
    Bitcoin,
    Solana,
    Ethereum,
    BnbSmartChain,
    Litecoin,
    Dogcoin,
    Sui,
    // Ton,
}

impl TryFrom<&str> for ChainCode {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let res = match value {
            crate::constant::chain_code::TRON => ChainCode::Tron,
            crate::constant::chain_code::BTC => ChainCode::Bitcoin,
            crate::constant::chain_code::SOLANA => ChainCode::Solana,
            crate::constant::chain_code::ETHEREUM => ChainCode::Ethereum,
            crate::constant::chain_code::BNB => ChainCode::BnbSmartChain,
            crate::constant::chain_code::LTC => ChainCode::Litecoin,
            crate::constant::chain_code::DOG => ChainCode::Dogcoin,
            crate::constant::chain_code::SUI => ChainCode::Sui,
            _ => return Err(crate::Error::UnknownChainCode),
        };
        Ok(res)
    }
}

impl std::fmt::Display for ChainCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainCode::Ethereum => write!(f, "{}", crate::constant::chain_code::ETHEREUM),
            ChainCode::Tron => write!(f, "{}", crate::constant::chain_code::TRON),
            ChainCode::Solana => write!(f, "{}", crate::constant::chain_code::SOLANA),
            ChainCode::BnbSmartChain => write!(f, "{}", crate::constant::chain_code::BNB),
            ChainCode::Bitcoin => write!(f, "{}", crate::constant::chain_code::BTC),
            ChainCode::Litecoin => write!(f, "{}", crate::constant::chain_code::LTC),
            ChainCode::Dogcoin => write!(f, "{}", crate::constant::chain_code::DOG),
            ChainCode::Sui => write!(f, "{}", crate::constant::chain_code::SUI),
            // ChainCode::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct ChainCodes(pub Vec<ChainCode>);

impl TryFrom<u32> for ChainCodes {
    type Error = crate::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let chain_code = match value {
            crate::constant::chain_type::ETH_TYPE => {
                vec![ChainCode::Ethereum, ChainCode::BnbSmartChain]
            }
            crate::constant::chain_type::TRON_TYPE => vec![ChainCode::Tron],
            crate::constant::chain_type::SOLANA_TYPE => vec![ChainCode::Solana],
            crate::constant::chain_type::BTC_TYPE => vec![ChainCode::Bitcoin],
            crate::constant::chain_type::LTC_TYPE => vec![ChainCode::Litecoin],
            crate::constant::chain_type::DOG_TYPE => vec![ChainCode::Dogcoin],
            crate::constant::chain_type::SUI_TYPE => vec![ChainCode::Sui],
            // crate::constant::chain_type::BTC_86_TYPE => vec![
            //     ChainCode::Btc,
            //     ChainCode::BtcTest,
            // ],
            _ => return Err(crate::Error::UnknownChainCode),
        };

        Ok(ChainCodes(chain_code))
    }
}
