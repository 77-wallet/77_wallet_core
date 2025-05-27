use std::str::FromStr as _;

pub fn alloy_address_from_str(address: &str) -> Result<alloy::primitives::Address, crate::Error> {
    address
        .parse::<alloy::primitives::Address>()
        .map_err(|e| crate::Error::Parse(e.into()))
}

pub fn sol_keypair_from_bytes(
    bytes: &[u8],
) -> Result<solana_sdk::signature::Keypair, crate::Error> {
    solana_sdk::signature::Keypair::from_bytes(bytes)
        .map_err(|e| crate::Error::Parse(crate::ParseError::SolanaSignatureError(e.to_string())))
}

pub fn method_from_str(method: &str) -> Result<http::Method, crate::Error> {
    reqwest::Method::from_str(method).map_err(|e| crate::Error::Parse(e.into()))
}

pub fn decimal_from_str(balance: &str) -> Result<rust_decimal::Decimal, crate::Error> {
    balance
        .parse::<rust_decimal::Decimal>()
        .map_err(|e| crate::Error::Parse(e.into()))
}

pub fn u64_from_str(balance: &str) -> Result<u64, crate::Error> {
    balance
        .parse::<u64>()
        .map_err(|e| crate::Error::Parse(e.into()))
}

pub fn f64_from_str(balance: &str) -> Result<f64, crate::Error> {
    balance
        .parse::<f64>()
        .map_err(|e| crate::Error::Parse(e.into()))
}

pub fn decode_from_percent(
    percent: percent_encoding::PercentDecode<'_>,
) -> Result<std::borrow::Cow<'_, str>, crate::Error> {
    percent
        .decode_utf8()
        .map_err(|e| crate::Error::Parse(e.into()))
}

pub fn derivation_path_percent_decode(
    encoded_derivation_path: &str,
) -> Result<std::borrow::Cow<'_, str>, crate::Error> {
    let percent_decode = percent_encoding::percent_decode_str(encoded_derivation_path);
    decode_from_percent(percent_decode)
}

pub fn derivation_path_percent_encode(
    raw_derivation_path: &str,
) -> percent_encoding::PercentEncode {
    percent_encoding::percent_encode(
        raw_derivation_path.as_bytes(),
        percent_encoding::NON_ALPHANUMERIC,
    )
}

pub fn parse_bech32_hrp(hrp: &str) -> Result<bech32::Hrp, crate::Error> {
    bech32::Hrp::parse(hrp).map_err(|e| crate::Error::Parse(e.into()))
}

type HmacSha512 = hmac::Hmac<sha2::Sha512>;
use hmac::Mac;

pub fn hmac_sha512(key: &[u8], data: &[u8]) -> Result<[u8; 64], crate::Error> {
    let mut hmac = HmacSha512::new_from_slice(key).map_err(|e| crate::Error::Parse(e.into()))?;
    hmac.update(data);
    let result = hmac.finalize();
    Ok(result.into_bytes().into())
}
