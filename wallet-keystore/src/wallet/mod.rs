pub mod phrase;
pub mod prikey;
pub mod seed;

pub trait WalletEncrypt: Sized {
    type Output;

    fn encrypt_keystore(self) -> Result<Self::Output, crate::Error>;
}

pub trait WalletDecrypt: Sized {
    type Output;

    fn decrypt_keystore(self) -> Result<Self::Output, crate::Error>;
}
