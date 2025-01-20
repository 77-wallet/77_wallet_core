use coins_bip32::prelude::k256::sha2::Sha512;
use coins_bip32::xkeys::XPriv;
use coins_bip39::{Mnemonic, MnemonicError};
use hmac::Hmac;
use pbkdf2::pbkdf2;
const PBKDF2_ROUNDS: u32 = 2048;
const PBKDF2_BYTES: usize = 64;

// 助记词->Mnemonic->root key
pub fn generate_master_key(
    language_code: u8,
    phrase: &str,
    password: &str,
) -> Result<(coins_bip32::xkeys::XPriv, Vec<u8>), crate::Error> {
    let wordlist_wrapper = crate::language::WordlistWrapper::new(language_code)?;
    Ok(match wordlist_wrapper {
        crate::language::WordlistWrapper::English(_) => {
            let mnemonic = Mnemonic::<coins_bip39::English>::new_from_phrase(phrase)?;
            let seed = mnemonic.to_seed(Some(password))?.to_vec();
            // let seed = seed;
            (mnemonic.master_key(Some(password))?, seed)
        }
        crate::language::WordlistWrapper::ChineseSimplified(_) => {
            let mnemonic = Mnemonic::<coins_bip39::ChineseSimplified>::new_from_phrase(phrase)?;
            let seed = mnemonic.to_seed(Some(password))?.to_vec();
            (mnemonic.master_key(Some(password))?, seed)
        }
        crate::language::WordlistWrapper::ChineseTraditional(_) => {
            let mnemonic = Mnemonic::<coins_bip39::ChineseTraditional>::new_from_phrase(phrase)?;
            let seed = mnemonic.to_seed(Some(password))?.to_vec();
            (mnemonic.master_key(Some(password))?, seed)
        }
        crate::language::WordlistWrapper::Czech(_) => {
            let mnemonic = Mnemonic::<coins_bip39::Czech>::new_from_phrase(phrase)?;
            let seed = mnemonic.to_seed(Some(password))?.to_vec();
            (mnemonic.master_key(Some(password))?, seed)
        }
        crate::language::WordlistWrapper::French(_) => {
            let mnemonic = Mnemonic::<coins_bip39::French>::new_from_phrase(phrase)?;
            let seed = mnemonic.to_seed(Some(password))?.to_vec();
            (mnemonic.master_key(Some(password))?, seed)
        }
        crate::language::WordlistWrapper::Italian(_) => {
            let mnemonic = Mnemonic::<coins_bip39::Italian>::new_from_phrase(phrase)?;
            let seed = mnemonic.to_seed(Some(password))?.to_vec();
            (mnemonic.master_key(Some(password))?, seed)
        }
        crate::language::WordlistWrapper::Japanese(_) => {
            let mnemonic = Mnemonic::<coins_bip39::Japanese>::new_from_phrase(phrase)?;
            let seed = mnemonic.to_seed(Some(password))?.to_vec();
            (mnemonic.master_key(Some(password))?, seed)
        }
        crate::language::WordlistWrapper::Korean(_) => {
            let mnemonic = Mnemonic::<coins_bip39::English>::new_from_phrase(phrase)?;
            let seed = mnemonic.to_seed(Some(password))?.to_vec();
            (mnemonic.master_key(Some(password))?, seed)
        }
        crate::language::WordlistWrapper::Portuguese(_) => {
            let mnemonic = Mnemonic::<coins_bip39::English>::new_from_phrase(phrase)?;
            let seed = mnemonic.to_seed(Some(password))?.to_vec();
            (mnemonic.master_key(Some(password))?, seed)
        }
        crate::language::WordlistWrapper::Spanish(_) => {
            let mnemonic = Mnemonic::<coins_bip39::English>::new_from_phrase(phrase)?;
            let seed = mnemonic.to_seed(Some(password))?.to_vec();
            (mnemonic.master_key(Some(password))?, seed)
        }
    })
}

/// 助记词 -> Mnemonic -> root key 不做语言校验
/// 直接使用输入的助记词和密码生成根密钥，不进行语言验证。
///
/// # 参数
/// - `phrase`: 用户提供的助记词
/// - `password`: 可选的密码，用于生成种子
///
/// # 返回
/// - 返回一个包含根密钥和种子的元组，如果有错误返回错误信息
pub fn generate_master_key_without_check(
    phrase: &str,
    password: &str,
) -> Result<(XPriv, Vec<u8>), crate::Error> {
    let seed = to_seed_v2(phrase, Some(password))?.to_vec();

    // 根据种子生成根私钥
    let master_key = XPriv::root_from_seed(seed.as_slice(), None)
        .map_err(coins_bip39::MnemonicError::Bip32Error)?;
    Ok((master_key, seed))
}

/// 将助记词和密码转换为种子（使用 PBKDF2 算法）。
/// 该方法不对助记词做校验。
///
/// # 参数
/// - `phrase`: 用户提供的助记词
/// - `password`: 可选的密码
///
/// # 返回
/// - 返回固定大小的种子数组
fn to_seed_v2(phrase: &str, password: Option<&str>) -> Result<[u8; PBKDF2_BYTES], MnemonicError> {
    let mut seed = [0u8; PBKDF2_BYTES];
    let salt = format!("mnemonic{}", password.unwrap_or(""));

    // 使用 PBKDF2 算法生成种子
    pbkdf2::<Hmac<Sha512>>(phrase.as_bytes(), salt.as_bytes(), PBKDF2_ROUNDS, &mut seed);

    Ok(seed)
}
