use coins_bip39::Mnemonic;

// 助记词->Mnemonic->root key
pub fn phrase_to_master_key(
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
