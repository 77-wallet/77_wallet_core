// #![feature(const_trait_impl, effects)]
pub(crate) mod crypto;
pub mod error;
pub mod keystore;
pub mod utils;
pub mod wallet;

pub use crate::error::Error;
// pub use service::Keystore;

pub use alloy::primitives::Address;
pub use keystore::builder::{KeystoreBuilder, RecoverableData};
pub use keystore::factory::KdfAlgorithm;
pub use keystore::json::KeystoreJson;
// pub use wallet_tree::wallet_tree::WalletTreeStrategy;

fn generate_random_bytes<R: rand::Rng + rand::CryptoRng>(rng: &mut R, len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    rng.fill_bytes(&mut bytes);
    bytes
}

/// Utility to get and set the chain ID on a transaction and the resulting signature within a
/// signer's `sign_transaction`.
#[macro_export]
macro_rules! sign_transaction_with_chain_code {
    // async (
    //    signer: impl Signer,
    //    tx: &mut impl SignableTransaction<Signature>,
    //    sign: lazy Signature,
    // )
    ($signer:expr, $tx:expr, $sign:expr) => {{
        if let Some(chain_code) = $signer.chain_code() {
            if !$tx.set_chain_code_checked(chain_code) {
                return Err(alloy::signers::Error::TransactionChainIdMismatch {
                    signer: chain_code,
                    // we can only end up here if the tx has a chain id
                    tx: $tx.chain_code().unwrap(),
                });
            }
        }

        let mut sig = $sign.map_err(alloy::signers::Error::other)?;

        if $tx.use_eip155() {
            if let Some(chain_code) = $signer.chain_code().or_else(|| $tx.chain_code()) {
                sig = sig.with_chain_code(chain_code);
            }
        }

        Ok(sig)
    }};
}
