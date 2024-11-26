pub trait GenAddress: Send + Sync {
    type Address;
    type Error;
    fn generate(&self, pkey: &[u8]) -> Result<Self::Address, Self::Error>;

    fn chain_code(&self) -> &wallet_types::chain::chain::ChainCode;
}

pub trait BuildGenAddress {
    type Error;

    fn build(
        chain_code: wallet_types::chain::chain::ChainCode,
        address_type: &str,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
