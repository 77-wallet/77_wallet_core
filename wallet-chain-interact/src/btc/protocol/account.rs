use crate::btc::utxos::Utxo;
use bitcoin::{Amount, ScriptBuf, TxOut};
use serde::Deserialize;

// used regiet network to scan out utxo
#[derive(Deserialize, Debug, Clone)]
pub struct ScanOut {
    pub unspents: Vec<ScanOutUtxo>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ScanOutUtxo {
    pub txid: String,
    pub vout: u32,
    pub amount: f64,
}

impl From<&ScanOutUtxo> for Utxo {
    fn from(value: &ScanOutUtxo) -> Self {
        Self {
            txid: value.txid.clone(),
            vout: value.vout,
            value: (value.amount * 100_000_000.0) as u64,
            confirmations: 6,
            selected: false,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct OutInfo {
    pub bestblock: String,
    pub confirmations: u64,
    pub value: f64,
    #[serde(rename = "scriptPubKey")]
    pub script_pubkey: ScriptPubkeyInfo,
    pub coinbase: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ScriptPubkeyInfo {
    pub hex: String,
}
impl TryFrom<OutInfo> for TxOut {
    type Error = crate::Error;
    fn try_from(value: OutInfo) -> Result<Self, Self::Error> {
        let script = ScriptBuf::from_hex(&value.script_pubkey.hex).unwrap();
        let amount = Amount::from_btc(value.value).unwrap();
        Ok(Self {
            value: amount,
            script_pubkey: script,
        })
    }
}
