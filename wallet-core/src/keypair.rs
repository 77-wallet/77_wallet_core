pub trait KeyPair {
    type Error;
    // type Address;
    // type Raw;
    // type PrivateKey;

    // fn master_key(&self) -> Result<coins_bip32::xkeys::XPriv, Self::Error>;

    fn chain_code(&self) -> wallet_types::chain::chain::ChainCode;

    // fn generate(seed: Vec<u8>, index: u32, chain_code: &ChainCode) -> Result<Self, Self::Error>
    // where
    //     Self: Sized;
    fn generate_with_derivation(
        seed: Vec<u8>,
        derivation_path: &str,
        chain_code: &wallet_types::chain::chain::ChainCode,
        network: wallet_types::chain::network::NetworkKind,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized;
    // fn public_key(&self) -> Vec<u8>;

    fn private_key(&self) -> Result<String, Self::Error>;

    fn pubkey(&self) -> String;

    fn network(&self) -> wallet_types::chain::network::NetworkKind;

    fn private_key_bytes(&self) -> Result<Vec<u8>, Self::Error>;
    // fn private_key<B>(&self) -> B
    // where
    //     B: AsRef<[u8]>;
    fn address(&self) -> String;

    fn derivation_path(&self) -> String;
}
