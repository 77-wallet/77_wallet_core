use wallet_types::{chain::address::r#type::DogAddressType, valueobject::AddressPubkey};
use wallet_utils::hex_func;

use crate::dog::utxos::Usedutxo;

pub struct MultisigAccountOpt {
    pub threshold: u8,
    pub owners: Vec<AddressPubkey>,
    pub address_type: DogAddressType,
}

impl MultisigAccountOpt {
    pub fn new(
        threshold: u8,
        owners: Vec<AddressPubkey>,
        address_type: &str,
    ) -> crate::Result<Self> {
        let address_type = DogAddressType::try_from(address_type)?;

        Ok(Self {
            threshold,
            owners,
            address_type,
        })
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct DogMultisigRaw {
    pub multisig_address: String,
    pub used_utxo: Usedutxo,
    pub raw_hex: String,
}
impl DogMultisigRaw {
    pub fn to_string(&self) -> crate::Result<String> {
        Ok(hex_func::bincode_encode(self)?)
        // let bytes = bincode::serialize(self).unwrap();
        // Ok(hex::encode(bytes))
    }

    pub fn from_hex_str(hex_str: &str) -> crate::Result<Self> {
        // let bytes = hex::decode(hex_str).unwrap();
        // Ok(bincode::deserialize::<BtcMultisigRaw>(&bytes).unwrap())
        Ok(hex_func::bincode_decode::<Self>(hex_str)?)
    }
}

pub struct MultisigTransactionOpt {
    pub from: String,
    pub value: String,
    pub script_hex: String,
    pub raw_data: String,
    pub address_type: DogAddressType,
}
impl MultisigTransactionOpt {
    pub fn new(
        from: String,
        value: String,
        script_hex: &str,
        raw_data: &str,
        address_type: &str,
    ) -> crate::Result<Self> {
        let address_type = DogAddressType::try_from(address_type)?;
        Ok(Self {
            from,
            value,
            script_hex: script_hex.to_string(),
            raw_data: raw_data.to_string(),
            address_type,
        })
    }
}
