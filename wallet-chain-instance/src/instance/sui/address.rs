#[derive(Clone)]
pub struct SuiGenAddress {}

impl wallet_core::address::GenAddress for SuiGenAddress {
    type Address = crate::instance::Address;
    type Error = crate::Error;

    fn generate(&self, pkey: &[u8]) -> Result<Self::Address, Self::Error> {
        Ok(crate::instance::Address::SolAddress(
            crate::instance::sol::secret_key_to_address(pkey).unwrap(),
        ))
    }

    fn chain_code(&self) -> &wallet_types::chain::chain::ChainCode {
        &wallet_types::chain::chain::ChainCode::Solana
    }
}
