#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    unreachable_pub,
    clippy::missing_const_for_fn,
    rustdoc::all
)]
#![deny(unused_must_use)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use std::fmt;

use crate::keystore::builder::RecoverableData;

pub(crate) mod builder;

/// A wallet instantiated with a locally stored private key
// pub type LocalWallet = PkWallet<k256::ecdsa::SigningKey>;

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
/// use alloy::signers::{Signer, SignerSync};
///
/// let wallet = alloy::signers_wallet::LocalWallet::random();
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
pub struct PkWallet {
    /// The wallet's private key.
    pub pkey: Vec<u8>,
    // /// The wallet's address.
    // pub address: wallet_chain_instance::instance::Address,
}

impl PkWallet {
    /// Construct a new wallet with an external [`PrehashSigner`].
    #[inline]
    pub fn new_with_signer(pkey: Vec<u8>) -> Self {
        PkWallet { pkey }
    }

    /// Returns this wallet's signer.
    #[inline]
    pub fn pkey(&self) -> Vec<u8> {
        self.pkey.clone()
    }

    // /// Returns this wallet's chain ID.
    // #[inline]
    // pub fn address(&self) -> wallet_chain_instance::instance::Address {
    //     self.address.clone()
    // }
}

impl PkWallet {
    /// Creates a new Wallet instance from a [`SigningKey`].
    ///
    /// This can also be used to create a Wallet from a [`SecretKey`](K256SecretKey).
    /// See also the `From` implementations.
    #[doc(alias = "from_private_key")]
    #[doc(alias = "new_private_key")]
    #[doc(alias = "new_pk")]
    #[inline]
    pub fn from_pkey(
        pkey: &[u8],
        // data: Box<
        //     dyn wallet_core::address::GenAddress<
        //         Address = wallet_chain_instance::instance::Address,
        //         Error = wallet_chain_instance::Error,
        //     >,
        // >,
    ) -> Result<Self, crate::Error> {
        // let address = data.generate(pkey)?;
        Ok(Self::new_with_signer(pkey.to_vec()))
    }

    /// Creates a new Wallet instance from a raw scalar serialized as a byte slice.
    ///
    /// Byte slices shorter than the field size (32 bytes) are handled by zero padding the input.
    #[inline]
    pub fn from_slice(
        pkey: &[u8],
        // data: Box<
        //     dyn wallet_core::address::GenAddress<
        //         Address = wallet_chain_instance::instance::Address,
        //         Error = wallet_chain_instance::Error,
        //     >,
        // >,
    ) -> Result<Self, crate::Error> {
        // Self::from_pkey(pkey, data)
        // let address = data.generate(pkey)?;
        Ok(Self::new_with_signer(pkey.to_vec()))
        // SigningKey::from_slice(bytes).map(|key| Self::from_signing_key(key, chain_code))
    }
}

// do not log the signer
// impl<D: PrehashSigner<(ecdsa::Signature, RecoveryId)>> fmt::Debug for PkWallet<D> {
impl fmt::Debug for PkWallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wallet").finish()
    }
}

impl PartialEq for PkWallet {
    fn eq(&self, other: &Self) -> bool {
        self.pkey.eq(&other.pkey)
    }
}

impl TryFrom<RecoverableData> for PkWallet {
    type Error = crate::Error;

    fn try_from(value: RecoverableData) -> Result<Self, Self::Error> {
        Ok(Self::from_slice(&value.inner())?)
    }
}
