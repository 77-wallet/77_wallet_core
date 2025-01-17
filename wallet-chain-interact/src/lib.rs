#![feature(let_chains)]
pub mod btc;
pub use btc::script;
pub mod ltc;
// pub use ltc as ltc_scripts;
pub mod eth;
pub mod factory;
pub mod sol;
pub mod tron;
mod utils;
pub use utils::*;
mod params;
pub use params::*;
mod errors;
pub use errors::*;
pub mod types;

pub use bitcoin::AddressType;
pub use litecoin::AddressType as LtcAddressType;
