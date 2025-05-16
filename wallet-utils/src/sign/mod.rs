use crate::error::sign_err::SignError;
use hex::decode;
use libsecp256k1::{Message, SecretKey};

pub fn sign_tron(s: &str, private_key: &str, recover: Option<u8>) -> Result<String, crate::Error> {
    let input = s.strip_prefix("0x").unwrap_or(s);

    let input_bytes = decode(input).map_err(|e| SignError::Message(e.to_string()))?;
    let message =
        Message::parse_slice(&input_bytes).map_err(|e| SignError::Message(e.to_string()))?;

    let private_key_bytes = decode(private_key).map_err(|e| SignError::KeyError(e.to_string()))?;
    let private_key = SecretKey::parse_slice(&private_key_bytes)
        .map_err(|e| SignError::KeyError(e.to_string()))?;

    let (signature, recovery_id) = libsecp256k1::sign(&message, &private_key);

    let mut full_signature = vec![0u8; 65];
    let mut id: u8 = recovery_id.into();
    if let Some(recover) = recover {
        id += recover
    }

    full_signature[..64].copy_from_slice(&signature.serialize());
    full_signature[64] = id;

    Ok(hex::encode(full_signature))
}

#[cfg(test)]
mod tests {
    use alloy::primitives::Keccak256;
    use hex::decode;
    use libsecp256k1::{Message, RecoveryId, Signature, recover};

    use crate::error::sign_err::SignError;

    fn verify_signature(message: &str, signature: &str) -> Result<bool, SignError> {
        let input_bytes = decode(message).map_err(|e| SignError::Message(e.to_string()))?;
        let signature_bytes = decode(signature).map_err(|e| SignError::Message(e.to_string()))?;
        // let public_key_bytes = decode(public_key).map_err(|e| SignError::KeyError(e.to_string()))?;

        // Parse the hashed message
        let message =
            Message::parse_slice(&input_bytes).map_err(|e| SignError::Message(e.to_string()))?;

        // Parse the signature
        let recovery_id = RecoveryId::parse(signature_bytes[64] - 27).unwrap();

        // let signature = Signature::parse_slice(&signature_bytes[..64]);
        let signature = Signature::parse_standard_slice(&signature_bytes[..64]).unwrap();

        // Recover the public key from the signature
        let recovered_public_key = recover(&message, &signature, &recovery_id).unwrap();

        let public_key_bytes = recovered_public_key.serialize();

        // Hash the public key bytes using Keccak256
        let mut hasher = Keccak256::new();
        hasher.update(&public_key_bytes[1..]); // Skip the first byte (format byte)
        let hash = hasher.finalize();
        // Take the last 20 bytes of the hash
        let address_bytes = &hash[12..];
        // Convert to hex string
        let addr = format!("0x{}", hex::encode(address_bytes));

        println!("address: {}", addr);
        // Verify if the recovered public key matches the given public key
        Ok(true)
    }

    #[test]
    fn test_verify() {
        let message = "069cce46b57b652b1d04ca2d74abe86b605d9d737879b138b631c43e3cb54328";
        let signature = "d146451393baaf172082c66f3f064d932c66059d2d99d2da3dd72fcdc2cd11597448883c4f18a729d007fcd1de1a0f999578f34e6bd25559da5b92cc6ac7a3581b";

        let aa = verify_signature(message, signature);

        println!("aa: {:?}", aa);
    }
}
