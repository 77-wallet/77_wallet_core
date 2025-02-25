/// 随机数生成器
use rand::{CryptoRng, Rng, RngCore as _};

pub trait SecureRng: Rng + CryptoRng {}

impl<T: Rng + CryptoRng> SecureRng for T {}

pub struct CryptoRandom {
    // 可以封装不同RNG实现
}

impl CryptoRandom {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_bytes(&mut self, len: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; len];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }
}
