#[derive(Clone)]
pub struct EthGenAddress {
    chain_code: wallet_types::chain::chain::ChainCode,
}

impl EthGenAddress {
    pub fn new(chain_code: wallet_types::chain::chain::ChainCode) -> Self {
        Self { chain_code }
    }
}

impl wallet_core::address::GenAddress for EthGenAddress {
    type Address = crate::instance::Address;
    type Error = crate::Error;
    fn generate(&self, pkey: &[u8]) -> Result<Self::Address, Self::Error> {
        let signer = alloy::signers::k256::ecdsa::SigningKey::from_slice(pkey).unwrap();
        Ok(crate::instance::Address::EthAddress(
            alloy::signers::utils::secret_key_to_address(&signer),
        ))
    }

    fn chain_code(&self) -> &wallet_types::chain::chain::ChainCode {
        &self.chain_code
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_() {
        let language = 1;
        let phrase = "fan swamp loop mesh enact tennis priority artefact canal hour skull joy";
        let password = "123";
        let (key, _) =
            wallet_core::xpriv::generate_master_key(language, &phrase, password).unwrap();

        let i: i32 = -1221;
        let index = if i < 0 {
            i.strict_add_unsigned(coins_bip32::BIP32_HARDEN) as u32
        } else {
            i as u32
        };
        println!("index: {index}");

        let path = "m/44h/60h/0h/0/2147482427h";

        // let derive = key.derive_child(index).unwrap();
        let derive = key.derive_path(path).unwrap();
        let signingkey: &coins_bip32::ecdsa::SigningKey = derive.as_ref();

        let address = alloy::signers::utils::secret_key_to_address(signingkey);
        println!("address: {address:?}");
    }
}
