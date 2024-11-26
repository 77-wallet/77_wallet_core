pub mod cbc;
pub mod ecb;

use base64::prelude::*;
use sha2::{Digest, Sha256};

pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

pub fn sha256_str(input: &str) -> Result<String, crate::Error> {
    let bytes = hex::decode(input).unwrap();
    Ok(hex::encode(sha256(&bytes)))
}

pub fn base58_encode(input: &[u8]) -> String {
    bs58::encode(input).into_string()
}

pub fn md5(input: &str) -> String {
    format!("{:x}", md5::compute(input))
}

pub fn md5_vec(input: &str) -> Vec<u8> {
    let hasher = md5::compute(input);
    hasher.as_slice().to_vec()
}

pub fn bytes_to_base64(input: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(input)
}
pub fn base64_to_bytes(input: &str) -> Result<Vec<u8>, crate::Error> {
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|e| crate::Error::Crypto(e.into()))
}

pub fn pbkdf2(
    password: &str,
    salt: &str,
    iterations: u32,
    output_len: usize,
) -> Result<Vec<u8>, crate::Error> {
    let mut output = vec![0u8; output_len];
    pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
        password.as_bytes(),
        salt.as_bytes(),
        iterations,
        &mut output,
    )
    .map_err(|e| crate::Error::Crypto(e.into()))?;
    Ok(output)
}

pub fn pbkdf2_string(
    password: &str,
    salt: &str,
    iterations: u32,
    output_len: usize,
) -> Result<String, crate::Error> {
    let mut output = vec![0u8; output_len];
    pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
        password.as_bytes(),
        salt.as_bytes(),
        iterations,
        &mut output,
    )
    .map_err(|e| crate::Error::Crypto(e.into()))?;

    Ok(hex::encode(sha256(&output)))
    // Ok(hex::encode(output))
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::init_test_log;

    #[test]
    fn test_sha256() {
        init_test_log();
        let str = "dlalkfja02034-012384019sodfaop[";
        let str = str.as_bytes();

        let rx = sha256(str);

        let hash = hex::encode(rx);
        tracing::info!("hash result = {:?}", hash);
        assert_eq!(
            hash,
            "5c8c657dcc57e1a82884d9c6e887788e91ff822e694e73e1e5838595eea813e3"
        );
    }

    #[test]
    fn test_base58_encode() {
        init_test_log();
        let input1 = "Hello";
        let digest1 = md5_vec(input1);
        // let str = str.as_bytes();
        let str = base58_encode(&digest1);
        tracing::info!("base58_encode result = {:?}", str);
    }
}
