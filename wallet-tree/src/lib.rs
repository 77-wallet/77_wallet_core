#![feature(trait_upcasting, let_chains)]
pub mod api;
pub mod error;
pub mod io;
pub mod layout;
pub mod migration;
pub mod naming;
pub mod service;
pub mod utils;
pub mod wallet_tree;

pub use error::Error;

pub use crate::wallet_tree::WalletTreeStrategy;
pub use service::Keystore;
pub use wallet_keystore::KdfAlgorithm;
