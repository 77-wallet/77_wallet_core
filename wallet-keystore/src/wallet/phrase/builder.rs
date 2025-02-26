use super::PhraseWallet;

pub(crate) struct PhraseEncryptorBuilder<'a, P, R, B, S> {
    keypath: P,
    rng: &'a mut R,
    data: B,
    password: S,
    name: Option<&'a str>,
    algorithm: crate::keystore::factory::KdfAlgorithm,
}

impl<'a, P, R, B, S> PhraseEncryptorBuilder<'a, P, R, B, S>
where
    P: AsRef<std::path::Path>,
    R: rand::Rng + rand::CryptoRng,
    B: AsRef<[u8]>,
    S: AsRef<[u8]>,
{
    pub(crate) fn new(
        keypath: P,
        rng: &'a mut R,
        data: B,
        password: S,
        name: Option<&'a str>,
        algorithm: crate::keystore::factory::KdfAlgorithm,
    ) -> Self {
        PhraseEncryptorBuilder {
            keypath,
            rng,
            data,
            password,
            name,
            algorithm,
        }
    }
}

impl<P, R, B, S> crate::wallet::WalletEncrypt for PhraseEncryptorBuilder<'_, P, R, B, S>
where
    P: AsRef<std::path::Path>,
    R: rand::Rng + rand::CryptoRng,
    B: AsRef<[u8]>,
    S: AsRef<[u8]>,
{
    type Output = (PhraseWallet, String);

    fn encrypt_keystore(self) -> Result<Self::Output, crate::Error> {
        let data = self.data.as_ref();
        let uuid = crate::crypto::encrypt_data(
            self.keypath,
            self.rng,
            data,
            self.password,
            self.name,
            self.algorithm,
        )?;
        let data = wallet_utils::conversion::vec_to_string(data)?;
        Ok((PhraseWallet::from_phrase(&data)?, uuid))
    }
}

pub(crate) struct PhraseDecryptorBuilder<P, S> {
    keypath: P,
    password: S,
}

impl<P, S> PhraseDecryptorBuilder<P, S>
where
    P: AsRef<std::path::Path>,
    S: AsRef<[u8]>,
{
    pub(crate) fn new(keypath: P, password: S) -> Self {
        PhraseDecryptorBuilder { keypath, password }
    }
}

impl<'a, P, S> crate::wallet::WalletDecrypt for PhraseDecryptorBuilder<P, S>
where
    P: AsRef<std::path::Path>,
    S: AsRef<[u8]>,
{
    type Output = PhraseWallet;

    fn decrypt_keystore(self) -> Result<Self::Output, crate::Error> {
        let phrase = crate::crypto::decrypt_data(self.keypath, self.password)?;
        let phrase = wallet_utils::conversion::vec_to_string(&phrase)?;
        Ok(PhraseWallet::from_phrase(&phrase)?)
    }
}
