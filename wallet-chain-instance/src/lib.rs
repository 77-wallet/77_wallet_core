#![feature(strict_overflow_ops)]
pub mod derivation_path;
pub mod error;
pub mod instance;

pub use error::{keypair::KeypairError, Error};
pub use instance::btc::address::generate_address_with_xpriv;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn add_index(path: &str, index: u32, hard: bool) -> String {
    let parts: Vec<&str> = path.rsplitn(2, '/').collect();
    if parts.len() != 2 {
        panic!("Invalid derivation path");
    }
    let index = if hard {
        format!("{index}'")
    } else {
        format!("{index}")
    };
    format!("{}/{}", parts[1], index)
}

pub fn add_solana_index(path: &str, index: u32) -> String {
    let parts: Vec<&str> = path.splitn(4, '\'').collect();
    if parts.len() != 4 {
        panic!("Invalid derivation path");
    }

    format!("{}'{}'/{}'{}", parts[0], parts[1], index, parts[3])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
