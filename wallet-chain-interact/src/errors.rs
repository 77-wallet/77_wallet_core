use chrono::TimeZone;
use regex::Regex;
use serde::de;
use thiserror::Error;

use crate::ton::errors::TonError;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum ParseErr {
    #[error("address parse error {0}")]
    AddressPraseErr(String),
    #[error("rpc url error {0}")]
    RpcUrlPraseErr(String),
    #[error("value convert error {0}")]
    ValueErr(String),
    #[error("tx hash error")]
    TxHashErr,
    #[error("json serialize{0}")]
    JsonErr(String),
    #[error("serde deserialize{0}")]
    SerdeErr(#[from] serde_json::Error),
    #[error("serialize {0}")]
    Serialize(String),
    #[error("sol multisig transaction account size parase error {0}")]
    SolMultisigArgs(String),
}

#[derive(Error, Debug)]
pub enum UtxoError {
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Insufficient fee need: {0}")]
    InsufficientFee(f64),
    #[error("Exceeds Maximum")]
    ExceedsMaximum,
    #[error("duct tx")]
    DustTx,
    #[error("Exceeds Max FeeRate")]
    ExceedsMaxFeeRate,
}

#[derive(Error, Debug)]
pub enum ContractValidationError {
    #[error("last withdraw time is {0}, less than 24 hours")]
    WithdrawTooSoon(String),
    #[error("witnessAccount does not have any reward")]
    WitnessAccountDoesNotHaveAnyReward,
    #[error("The lock period for ENERGY this time cannot be less than the remaining time {0}ms")]
    EnergyLockPeriodTooShort(i64),
    #[error("{0}")]
    Other(String),
}

impl<'de> serde::Deserialize<'de> for ContractValidationError {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TronErrorVisitor;

        impl<'de> de::Visitor<'de> for TronErrorVisitor {
            type Value = ContractValidationError;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "a string representing a contract validation error"
                )
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                let withdraw_regex =
                    Regex::new(r"last withdraw time is (\d+), less than 24 hours").unwrap();
                let balance_regex = Regex::new(r"witnessAccount does not have any reward").unwrap();

                // The lock period for ENERGY this time cannot be less than the remaining time[259092000ms] of the last lock period for ENERGY!
                let energy_lock_period_too_short_regex = Regex::new(r"The lock period for ENERGY this time cannot be less than the remaining time\[(\d+)ms\]").unwrap();
                if let Some(caps) = withdraw_regex.captures(value) {
                    let timestamp = caps[1].parse::<i64>().unwrap();
                    // 把时间戳转成 Utc时间
                    let time = chrono::Utc.timestamp_micros(timestamp).unwrap().to_string();
                    Ok(ContractValidationError::WithdrawTooSoon(time))
                } else if let Some(caps) = energy_lock_period_too_short_regex.captures(value) {
                    let timestamp = caps[1].parse::<i64>().unwrap();
                    Ok(ContractValidationError::EnergyLockPeriodTooShort(timestamp))
                } else if balance_regex.is_match(value) {
                    Ok(ContractValidationError::WitnessAccountDoesNotHaveAnyReward)
                } else {
                    Ok(ContractValidationError::Other(value.to_string()))
                }
            }
        }

        deserializer.deserialize_str(TronErrorVisitor)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    TransportError(#[from] wallet_transport::errors::TransportError),
    #[error("rpc node return error: {0}")]
    RpcNode(String),
    #[error("utils error {0}")]
    UtilsError(#[from] wallet_utils::error::Error),
    #[error("parse error {0}")]
    AbiParseError(String),
    #[error("types error {0}")]
    Types(#[from] wallet_types::Error),
    #[error("Ton error {0}")]
    TonError(#[from] TonError),
    // flow to optimize
    #[error("hex error {0}")]
    HexError(String),
    #[error("btc script error {0}")]
    BtcScript(String),
    #[error("sign error {0}")]
    SignError(String),
    #[error("{0}")]
    Other(String),
    #[error("not support api:{0}")]
    NotSupportApi(String),
    #[error("rpc error {0}")]
    RpcError(String),
    #[error("contract validation error {0}")]
    ContractValidationError(ContractValidationError),
    #[error("parse error {0}")]
    ParseError(#[from] ParseErr),
    #[error("utxo error")]
    UtxoError(#[from] UtxoError),
    #[error("transfer error {0}")]
    TransferError(String),
    #[error("any chain")]
    AnyChainError(#[from] anychain_core::error::Error),
    #[error("any chain transaction")]
    AnyTransaction(#[from] anychain_core::TransactionError),
}

impl Error {
    pub fn is_network_error(&self) -> bool {
        match self {
            Error::TransportError(e) => e.is_network_error(),
            Error::UtilsError(e) => e.is_network_error(),
            _ => false,
        }
    }
}
