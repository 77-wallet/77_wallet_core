pub mod account;
pub mod args;
pub mod compiled_keys;
pub mod pda;
pub mod program;
pub mod small_vec;
pub mod transfer;
pub mod vault_transaction;

pub const MULTISIG_PROGRAM_ID: &str = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf";

pub fn get_selector(method: &str) -> Vec<u8> {
    let global = "global:";

    let discriminator = format!("{}{}", global, method);
    let command = wallet_utils::sha256(discriminator.as_bytes());

    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&command[..8]);
    discriminator.to_vec()
}
