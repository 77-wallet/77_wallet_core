pub const ETH_DERIVATION_PATH: &str = "m/44'/60'/0'/0/0";
pub const ETH_HARD_DERIVATION_PATH: &str = "m/44'/60'/0'/0/0";

pub const TRON_DERIVATION_PATH: &str = "m/44'/195'/0'/0/0";
pub const TRON_HARD_DERIVATION_PATH: &str = "m/44'/195'/0'/0/0";

pub const SOLANA_DERIVATION_PATH: &str = "m/44'/501'/0'/0";

/// legacy
pub const BTC_DERIVATION_PATH: &str = "m/44'/0'/0'/0/0";
pub const BTC_HARD_DERIVATION_PATH: &str = "m/44'/0'/0'/0/0";

/// 隔离见证（原生）
pub const BTC_SEG_WIT_NATIVE_DERIVATION_PATH: &str = "m/84'/0'/0'/0/0";
pub const BTC_SEG_WIT_NATIVE_HARD_DERIVATION_PATH: &str = "m/84'/0'/0'/0/0";

/// Taproot
pub const BTC_TAPROOT_DERIVATION_PATH: &str = "m/86'/0'/0'/0/0";
pub const BTC_TAPROOT_HARD_DERIVATION_PATH: &str = "m/86'/0'/0'/0/0";
/// 隔离见证（兼容）
pub const BTC_SEG_WIT_DERIVATION_PATH: &str = "m/49'/0'/0'/0/0";
pub const BTC_SEG_WIT_HARD_DERIVATION_PATH: &str = "m/49'/0'/0'/0/0";

pub const BTC_86_TEST_DERIVATION_PATH: &str = "m/86'/1'/0'/0/0";

/// legacy
pub const LTC_DERIVATION_PATH: &str = "m/44'/0'/0'/0/0";
pub const LTC_HARD_DERIVATION_PATH: &str = "m/44'/0'/0'/0/0";

/// 隔离见证（原生）
pub const LTC_SEG_WIT_NATIVE_DERIVATION_PATH: &str = "m/84'/0'/0'/0/0";
pub const LTC_SEG_WIT_NATIVE_HARD_DERIVATION_PATH: &str = "m/84'/0'/0'/0/0";

/// Taproot
pub const LTC_TAPROOT_DERIVATION_PATH: &str = "m/86'/0'/0'/0/0";
pub const LTC_TAPROOT_HARD_DERIVATION_PATH: &str = "m/86'/0'/0'/0/0";
/// 隔离见证（兼容）
pub const LTC_SEG_WIT_DERIVATION_PATH: &str = "m/49'/0'/0'/0/0";
pub const LTC_SEG_WIT_HARD_DERIVATION_PATH: &str = "m/49'/0'/0'/0/0";

pub const LTC_86_TEST_DERIVATION_PATH: &str = "m/86'/1'/0'/0/0";

pub const ETH_DERIVATION_PATH_START: &str = "m/44'/60'/";
pub const TRON_DERIVATION_PATH_START: &str = "m/44'/195'/";
pub const SOLANA_DERIVATION_PATH_START: &str = "m/44'/501'/";

pub mod chain_type {
    pub const ETH_TYPE: u32 = 60;
    pub const TRON_TYPE: u32 = 195;
    pub const SOLANA_TYPE: u32 = 501;
    pub const BTC_TYPE: u32 = 0;
    pub const BTC_86_TYPE: u32 = 86;
}

pub mod chain_code {
    pub const ETHEREUM: &str = "eth";
    pub const TRON: &str = "tron";
    pub const SOLANA: &str = "sol";
    pub const BNB: &str = "bnb";
    pub const BTC: &str = "btc";
    pub const LTC: &str = "ltc";
}

pub mod coin {
    pub const ETH: &str = "eth";
    pub const TRX: &str = "trx";
    pub const USDT: &str = "usdt";
    pub const JST: &str = "jst";
}

pub mod token_address {
    pub const TRX_TOKEN: &str = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";
    pub const USDT_TOKEN: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
    pub const JST_TOKEN: &str = "TF17BgPaZYbz8oxbjhriubPDsA7ArKoLX3";
}

pub mod decimals {
    pub const ETH_DECIMALS: u8 = 18;
    pub const SOL_DECIMALS: u8 = 9;
    pub const USDT_DECIMALS: u8 = 6;
    pub const TRX_DECIMALS: u8 = 6;
    pub const JST_DECIMALS: u8 = 18;
}

pub mod btc_address_type {
    pub const P2PKH: &str = "p2pkh";
    pub const P2SH: &str = "p2sh";
    pub const P2SH_WPKH: &str = "p2sh-wpkh";
    pub const P2SH_WSH: &str = "p2sh-wsh";
    pub const P2WPKH: &str = "p2wpkh";
    pub const P2WSH: &str = "p2wsh";
    pub const P2TR: &str = "p2tr";
    pub const P2TR_SH: &str = "p2tr-sh";
}

pub mod btc_address_catecory {
    pub const TAPROOT: &str = "Taproot";
    pub const NESTED_SEG_WIT: &str = "Nested SegWit";
    pub const NATIVE_SEG_WIT: &str = "Native SegWit";
    pub const LEGACY: &str = "Legacy";
}

// 定义哪些合约需要验证是否具有黑明单,目前只有usdt
pub mod check_black {
    pub const ETH: &[&str] = &["0xdAC17F958D2ee523a2206206994597C13D831ec7"];
    pub const SOLANA: &[&str] = &["Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"];
    pub const BNB: &[&str] = &[];
    pub const TRON: &[&str] = &["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"];
    pub const BTC: &[&str] = &[];
}
