use alloy::primitives::{Address, U256};
use alloy::sol_types::SolValue;

// 地址进行abi编码
pub fn abi_encode_address(addr: &str) -> String {
    let addr_bytes = hex::decode(addr).unwrap();

    let address_bytes = &addr_bytes[1..21];
    let address = Address::from_slice(address_bytes);
    let abi = address.abi_encode();
    hex::encode(abi)
}

// u256 abi编码
pub fn abi_encode_u256(value: U256) -> String {
    let abi = value.abi_encode();
    hex::encode(abi)
}

#[test]
fn test_abi_encode_address() {
    let addr = "0000000000000000000000002cbe86ef1ace939f8eb22a722412260abcc7b6a3".to_string();
    let bytes = hex::decode(addr).unwrap();

    let c = Address::abi_decode(&bytes, true).unwrap();
    println!("cc {:?}", c);
}
