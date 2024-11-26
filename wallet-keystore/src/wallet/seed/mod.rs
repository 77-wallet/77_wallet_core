#![warn(unreachable_pub, clippy::missing_const_for_fn, rustdoc::all)]
#![deny(unused_must_use)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use std::fmt;

use crate::error::wallet::WalletError;

pub(crate) mod builder;

/// An Ethereum private-public key pair which can be used for signing messages.
///
/// # Examples
///
/// ## Signing and Verifying a message
///
/// The wallet can be used to produce ECDSA [`Signature`] objects, which can be
/// then verified. Note that this uses
/// [`eip191_hash_message`](alloy_primitives::eip191_hash_message) under the hood which will
/// prefix the message being hashed with the `Ethereum Signed Message` domain separator.
///
/// ```
/// use alloy_signer::{Signer, SignerSync};
///
/// let wallet = alloy_signer_wallet::LocalWallet::random();
///
/// // Optionally, the wallet's chain id can be set, in order to use EIP-155
/// // replay protection with different chains
/// let wallet = wallet.with_chain_code(Some(1337));
///
/// // The wallet can be used to sign messages
/// let message = b"hello";
/// let signature = wallet.sign_message_sync(message)?;
/// assert_eq!(signature.recover_address_from_msg(&message[..]).unwrap(), wallet.address());
///
/// // LocalWallet is clonable:
/// let wallet_clone = wallet.clone();
/// let signature2 = wallet_clone.sign_message_sync(message)?;
/// assert_eq!(signature, signature2);
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct SeedWallet {
    /// The wallet's private key.
    pub seed: Vec<u8>,
}

impl SeedWallet {
    /// Construct a new wallet with an external [`PrehashSigner`].
    #[inline]
    pub const fn new_with_seed(seed: Vec<u8>) -> Self {
        SeedWallet { seed }
    }

    /// Returns this wallet's signer.
    #[allow(unused)]
    #[inline]
    pub const fn seed(&self) -> &Vec<u8> {
        &self.seed
    }

    /// Consumes this wallet and returns its signer.
    #[inline]
    pub fn into_seed(self) -> Vec<u8> {
        self.seed
    }
}

impl SeedWallet {
    /// Creates a new Wallet instance from a [`SigningKey`].
    ///
    /// This can also be used to create a Wallet from a [`SecretKey`](K256SecretKey).
    /// See also the `From` implementations.
    #[inline]
    pub(crate) fn from_seed(seed: Vec<u8>) -> Result<Self, WalletError> {
        // let pri_key = XPriv::root_from_seed(seed.as_slice(), None)?;
        // let signingkey: &coins_bip32::ecdsa::SigningKey = pri_key.as_ref();

        // let address = secret_key_to_address(signingkey);
        Ok(Self::new_with_seed(seed))
    }
}

// do not log the signer
impl fmt::Debug for SeedWallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wallet").finish()
    }
}
impl PartialEq for SeedWallet {
    fn eq(&self, other: &Self) -> bool {
        self.seed.eq(&other.seed)
        //  && self.address == other.address
    }
}
