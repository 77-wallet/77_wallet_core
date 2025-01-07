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

// 多签程序是否进行创建了账号
pub fn sods4_v4_has_create_account(data: &str) -> bool {
    let bytes = solana_sdk::bs58::decode(data)
        .into_vec()
        .unwrap_or_default();
    let target = &bytes[0..8];

    match target {
        // 创建交易
        [48, 250, 78, 168, 208, 226, 218, 211] => true,
        // 创建投票账号
        [220, 60, 73, 224, 30, 108, 79, 159] => true,
        // 创建多签账号
        [50, 221, 199, 93, 40, 245, 139, 233] => true,
        _ => false,
    }
}

// #[test]
// fn tss_() {
//     let a = get_selector("proposal_create");
//     println!("{:?}", a);

//     let a = get_selector("multisig_create_v2");
//     println!("{:?}", a);
// }
