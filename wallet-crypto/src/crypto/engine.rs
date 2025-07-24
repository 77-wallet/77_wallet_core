use rand::{CryptoRng, Rng};

pub trait CryptoEngine {
    type Data;

    fn encrypt<T: AsRef<[u8]>, R: Rng + CryptoRng>(
        &self,
        rng: &mut R,
        data: &T,
        password: &[u8],
    ) -> Result<Self::Data, crate::Error>;

    fn decrypt(&self, password: &[u8], keystore: Self::Data) -> Result<Vec<u8>, crate::Error>;
}
