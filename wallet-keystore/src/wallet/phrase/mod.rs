#![warn(unreachable_pub, clippy::missing_const_for_fn, rustdoc::all)]
#![deny(unused_must_use)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use std::fmt;

use serde::Deserialize;

use crate::error::wallet::WalletError;

pub(crate) mod builder;
#[derive(Clone, Deserialize)]
pub struct PhraseWallet {
    /// The wallet's private key.
    pub phrase: String,
}

impl PhraseWallet {
    /// Construct a new wallet with an external [`PrehashSigner`].
    #[inline]
    pub const fn new_with_phrase(phrase: String) -> Self {
        PhraseWallet { phrase }
    }

    /// Returns this wallet's signer.
    #[allow(unused)]
    #[inline]
    pub const fn phrase(&self) -> &String {
        &self.phrase
    }
}

impl PhraseWallet {
    /// Creates a new Wallet instance from a [`SigningKey`].
    ///
    /// This can also be used to create a Wallet from a [`SecretKey`](K256SecretKey).
    /// See also the `From` implementations.
    #[inline]
    pub(crate) fn from_phrase(phrase: &str) -> Result<Self, WalletError> {
        Ok(Self::new_with_phrase(phrase.to_string()))
    }
}

// do not log the signer
impl fmt::Debug for PhraseWallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wallet")
            .field("phrase", &self.phrase)
            .finish()
    }
}

impl PartialEq for PhraseWallet {
    fn eq(&self, other: &Self) -> bool {
        self.phrase.eq(&other.phrase) && self.phrase == other.phrase
    }
}
