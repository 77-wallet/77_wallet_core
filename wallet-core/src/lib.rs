pub mod address;
pub mod derive;
pub mod error;
pub mod keypair;
pub mod language;
pub mod xpriv;

pub use crate::error::Error;
pub use keypair::KeyPair;

#[cfg(test)]
mod tests {
    use solana_sdk::signer::Signer;

    #[test]
    fn test() {
        // solana_sdk::signature::
        // solana_sdk::signature::

        let key = solana_sdk::signature::Keypair::from_base58_string("4JsE64VDotmqJMJRQz53XDAxVCwhmm7bdhybNVMA4fu12WB2etB3QgPwfA3FUFGf2M8RzN9NLF9cJX1sV5ygpQvP");

        let secret = key.to_base58_string();
        println!("Solana secret: {:?}", secret);
        let secret = key.secret();
        println!("Solana secret: {:?}", secret);

        let pubkey = key.pubkey();
        // // 假设您有一个32字节的公钥数据
        // let public_key_bytes: [u8; 32] = [
        //     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
        //     24, 25, 26, 27, 28, 29, 30, 31,
        // ];

        // // 使用公钥字节数组创建一个Pubkey对象
        // let pubkey = solana_sdk::pubkey::Pubkey::new(&public_key_bytes);

        // 获取Solana地址（即Base58编码的字符串表示）
        let solana_address = pubkey.to_string();

        // 打印Solana地址
        println!("Solana Address: {}", solana_address);
    }
}
