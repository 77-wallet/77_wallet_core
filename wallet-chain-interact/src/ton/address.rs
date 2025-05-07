use super::errors::TonError;
use tonlib_core::TonAddress;

pub fn parse_addr_from_bs64_url(add: &str) -> Result<TonAddress, TonError> {
    Ok(TonAddress::from_base64_url(add)?)
}
