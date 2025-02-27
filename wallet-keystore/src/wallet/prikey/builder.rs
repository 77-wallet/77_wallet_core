use super::PkWallet;

pub(crate) struct PrikeyEncryptorBuilder<'a, P, R, B, S> {
    keypath: P,
    rng: &'a mut R,
    pk: B,
    password: S,
    name: Option<&'a str>,
    // data: Box<
    //     dyn wallet_core::address::GenAddress<
    //         Address = wallet_chain_instance::instance::Address,
    //         Error = wallet_chain_instance::Error,
    //     >,
    // >,
    algorithm: crate::keystore::factory::KdfAlgorithm,
}

impl<'a, P, R, B, S> PrikeyEncryptorBuilder<'a, P, R, B, S>
where
    P: AsRef<std::path::Path>,
    R: rand::Rng + rand::CryptoRng,
    B: AsRef<[u8]>,
    S: AsRef<[u8]>,
{
    pub(crate) fn new(
        keypath: P,
        rng: &'a mut R,
        pk: B,
        password: S,
        name: Option<&'a str>,
        // data: Box<
        //     dyn wallet_core::address::GenAddress<
        //         Address = wallet_chain_instance::instance::Address,
        //         Error = wallet_chain_instance::Error,
        //     >,
        // >,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Self {
        PrikeyEncryptorBuilder {
            keypath,
            rng,
            pk,
            password,
            name,
            // data,
            algorithm,
        }
    }
}

#[async_trait::async_trait]
impl<P, R, B, S> crate::wallet::WalletEncrypt for PrikeyEncryptorBuilder<'_, P, R, B, S>
where
    P: AsRef<std::path::Path>,
    R: rand::Rng + rand::CryptoRng,
    B: AsRef<[u8]>,
    S: AsRef<[u8]>,
{
    type Output = PkWallet;

    fn encrypt_keystore(self) -> Result<Self::Output, crate::Error> {
        let data = self.pk.as_ref();
        crate::crypto::encrypt_data(
            self.keypath,
            self.rng,
            data,
            self.password,
            self.name,
            self.algorithm,
        )?;
        Ok(PkWallet::from_slice(data)?)
    }
}

pub(crate) struct PrikeyDecryptorBuilder<P, S> {
    keypath: P,
    password: S,
}

impl<P, S> PrikeyDecryptorBuilder<P, S>
where
    P: AsRef<std::path::Path>,
    S: AsRef<[u8]>,
{
    pub(crate) fn new(keypath: P, password: S) -> Self {
        PrikeyDecryptorBuilder { keypath, password }
    }
}

impl<'a, P, S> crate::wallet::WalletDecrypt for PrikeyDecryptorBuilder<P, S>
where
    P: AsRef<std::path::Path>,
    S: AsRef<[u8]>,
{
    type Output = PkWallet;

    fn decrypt_keystore(self) -> Result<Self::Output, crate::Error> {
        let secret = crate::crypto::decrypt_data(self.keypath, self.password)?;
        PkWallet::from_slice(&secret)
    }
}
