use litecoin::TxIn;
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr as _};

pub type Usedutxo = HashMap<String, Utxo>;
pub struct UtxoList(pub Vec<Utxo>);

impl UtxoList {
    pub fn inputs_from_utxo(&mut self, amount: litecoin::Amount) -> crate::Result<Vec<TxIn>> {
        let mut total_value = litecoin::Amount::default();

        let mut inputs = Vec::new();
        for tx in self.0.iter_mut() {
            total_value += litecoin::Amount::from_sat(tx.value);
            tx.selected = true;
            inputs.push(TxIn::from(tx.clone()));

            if total_value >= amount {
                break;
            }
        }

        if total_value < amount {
            return Err(crate::UtxoError::InsufficientBalance.into());
        }

        Ok(inputs)
    }

    // select all utxo
    pub fn selected_all(&mut self) -> crate::Result<Vec<TxIn>> {
        let inputs: Vec<TxIn> = self
            .0
            .iter_mut()
            .map(|tx| {
                tx.selected = true;
                TxIn::from(tx.clone())
            })
            .collect();

        Ok(inputs)
    }

    pub fn total_input_amount(&self) -> litecoin::Amount {
        self.0
            .iter()
            .filter(|u| u.selected)
            .map(|item| litecoin::Amount::from_sat(item.value))
            .sum()
    }

    pub fn used_utxo_to_hash_map(&self) -> Usedutxo {
        let mut result = HashMap::new();

        self.0.iter().filter(|item| item.selected).for_each(|item| {
            let key = format!("{}-{}", item.txid, item.vout);
            result.insert(key, item.clone());
        });

        result
    }

    pub fn available_utxo(&self) -> Vec<Utxo> {
        self.0.iter().filter(|u| !u.selected).cloned().collect()
    }

    /// tag utxo is used
    pub fn tag_select(&mut self, tx_id: &str, vout: u32) {
        if let Some(item) = self
            .0
            .iter_mut()
            .find(|item| item.txid == tx_id && item.vout == vout)
        {
            item.selected = true;
        }
    }

    pub fn balance(&self) -> u64 {
        self.0.iter().map(|item| item.value).sum()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    #[serde(deserialize_with = "value_to_u64", serialize_with = "u64_to_string")]
    // the value unit is sat
    pub value: u64,
    pub confirmations: u32,
    // Custom field representing whether it is selected. default is not
    #[serde(default)]
    pub selected: bool,
}

fn value_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    // 将字符串解析为 u64
    s.parse::<u64>()
        .map_err(|e| serde::de::Error::custom(format!("Failed to parse string as u64: {}", e)))
}

fn u64_to_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

// utxo to transaction tx_in
impl From<Utxo> for litecoin::TxIn {
    fn from(value: Utxo) -> Self {
        Self {
            previous_output: litecoin::OutPoint {
                txid: litecoin::Txid::from_str(&value.txid).unwrap(),
                vout: value.vout,
            },
            script_sig: litecoin::ScriptBuf::default(),
            sequence: litecoin::Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: litecoin::Witness::default(),
        }
    }
}

#[test]
fn test_utxo() {
    let str = r#"[{"txid":"a14193ef8c74dd19c3368d371a1958099bd499e47f3eeaf5480db41da43e7547","vout":1,"value":"90352","height":2874324,"confirmations":29640},{"txid":"e0dc515b0fc89513335fe42c31ad60616b2e1ccfe2c4924da76eee2ba48ef969","vout":0,"value":"200000","height":2867555,"confirmations":36409},{"txid":"0df541af055bb5e677c34692fa7c3cb327ec9051bb344c7aac831aae048e7ee9","vout":3,"value":"200000","height":2866192,"confirmations":37772},{"txid":"174a9e102ccd3c63a570e22ea00a66098f4b6af006bed5fc2fdfed28997c01c8","vout":0,"value":"150000","height":2865692,"confirmations":38272},{"txid":"28a42516124595dbe4a146ffc71a9ddeff2fbe3ec470a65ea9dad5d9786ea1ef","vout":1,"value":"200000","height":2865304,"confirmations":38660},{"txid":"d5f10fe9b1957670328baf6f9cb6e0afa8e7e0b0f424ef3439a6ebfb200742c9","vout":14,"value":"250000","height":2822237,"confirmations":81727}]"#;
    let res = serde_json::from_str::<Vec<Utxo>>(str).unwrap();
    println!("{:?}", res);
}

// struct U64OrStringVisitor;
// impl<'de> Visitor<'de> for U64OrStringVisitor {
//     type Value = u64;

//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         formatter.write_str("a u64 or a string containing a u64")
//     }

//     fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
//     where
//         E: de::Error,
//     {
//         Ok(value)
//     }

//     fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
//     where
//         E: de::Error,
//     {
//         value
//             .parse::<u64>()
//             .map_err(|e| de::Error::custom(format!("Failed to parse string as u64: {}", e)))
//     }
// }
