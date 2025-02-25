use digest::Update as _;
use sha2::Digest as _;
use sha3::Keccak256;

pub trait MacCalculator {
    fn compute(&self, key: &[u8], data: &[u8]) -> Vec<u8>;
}

pub struct Keccak256Mac;

impl MacCalculator for Keccak256Mac {
    fn compute(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        Keccak256::new()
            .chain(&key[16..32])
            .chain(data)
            .finalize()
            .to_vec()
    }
}