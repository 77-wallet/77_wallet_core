#[derive(Clone)]
pub struct TrxGenAddress {}

impl wallet_core::address::GenAddress for TrxGenAddress {
    type Address = crate::instance::Address;
    type Error = crate::Error;

    fn generate(&self, pkey: &[u8]) -> Result<Self::Address, Self::Error> {
        let signer = alloy::signers::k256::ecdsa::SigningKey::from_slice(pkey).unwrap();
        Ok(crate::instance::Address::TrxAddress(
            crate::instance::trx::secret_key_to_address(&signer)?,
        ))
    }

    fn chain_code(&self) -> &wallet_types::chain::chain::ChainCode {
        &wallet_types::chain::chain::ChainCode::Tron
    }
}

#[cfg(test)]
mod test {
    use anychain_core::Address as _;

    #[test]
    fn test_() {
        let language = 1;
        let phrase =
            "member diesel marine culture boat differ spirit patient drum fix chunk sadness";
        let password = "1234qwer";
        let (key, _) =
            wallet_core::xpriv::generate_master_key(language, &phrase, password).unwrap();

        // let i: i32 = -1221;
        // let index = if i < 0 {
        //     i.strict_add_unsigned(coins_bip32::BIP32_HARDEN) as u32
        // } else {
        //     i as u32
        // };
        // println!("index: {index}");
        // let index = 0;
        let path = "m/44h/195h/0h/0/0";

        let derive = key.derive_path(path).unwrap();

        let signingkey: &coins_bip32::ecdsa::SigningKey = derive.as_ref();
        let private_key = signingkey.to_bytes();

        let private_key = libsecp256k1::SecretKey::parse_slice(&private_key).unwrap();

        let address = anychain_tron::TronAddress::from_secret_key(
            &private_key,
            &anychain_tron::TronFormat::Standard,
        )
        .unwrap();
        println!("address: {address:?}");
    }
}
