use super::errors::TonError;
use tonlib_core::TonAddress;

pub fn parse_addr_from_bs64_url(add: &str) -> Result<TonAddress, TonError> {
    Ok(TonAddress::from_base64_url(add)?)
}

pub fn hash_to_hex(hash: &str) -> crate::Result<String> {
    let bytes = wallet_utils::base64_to_bytes(hash)?;

    Ok(wallet_utils::hex_func::hex_encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_to_hex() {
        let hash = "HbFgmAxhX07vD7cD2aYu5NkIwEjxtlg4MvcG8AfmFV4=";

        let res = hash_to_hex(hash).unwrap();
        assert_eq!(
            "1db160980c615f4eef0fb703d9a62ee4d908c048f1b6583832f706f007e6155e",
            res
        )
    }
}
