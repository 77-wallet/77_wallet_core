pub trait Derive {
    type Error;
    type Item: crate::KeyPair;

    // fn derive(&self, seed: Vec<u8>, index: u32) -> Result<coins_bip32::xkeys::XPriv, Self::Error>;
    // fn derive(&self, seed: Vec<u8>, index: u32) -> Result<Self::Item, Self::Error>;

    fn derive_with_derivation_path(
        &self,
        seed: Vec<u8>,
        derivation_path: &str,
    ) -> Result<Self::Item, Self::Error>;
}

pub trait GenDerivation {
    type Error;
    fn generate(
        address_type: &Option<wallet_types::chain::address::r#type::BtcAddressType>,
        input_index: i32,
    ) -> Result<String, Self::Error>;
}

pub trait GenDerivationLtc {
    type Error;
    fn generate(
        address_type: &Option<wallet_types::chain::address::r#type::LtcAddressType>,
        input_index: i32,
    ) -> Result<String, Self::Error>;
}

// pub fn derive(seed: Vec<u8>, index: u32) -> Result<coins_bip32::xkeys::XPriv, crate::Error> {
//     let pri_key = coins_bip32::xkeys::XPriv::root_from_seed(seed.as_slice(), None).unwrap();
//     let path = crate::constant::add_index(crate::constant::ETH_DERIVATION_PATH, index);
//     // TODO:
//     let derive = pri_key.derive_path(path.as_str()).unwrap();
//     Ok(derive)
// }
