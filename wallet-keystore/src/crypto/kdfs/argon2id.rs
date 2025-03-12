use crate::{
    error::crypto::KeystoreError,
    keystore::factory::{Argon2idParams, KdfParams},
    KdfAlgorithm,
};

use super::KeyDerivationFunction;

pub struct Argon2idKdf {
    pub params: Argon2idParams,
}

impl Argon2idKdf {
    pub fn new(params: Argon2idParams) -> Self {
        Self { params }
    }

    /// 推荐的安全参数（根据 OWASP 2023 建议）
    pub fn recommended_params<R: rand::RngCore>(rng: &mut R) -> Self {
        let salt = {
            let mut salt = vec![0u8; 16];
            rng.fill_bytes(&mut salt);
            salt
        };

        Self {
            params: Argon2idParams {
                dklen: 32,    // 32字节密钥
                time_cost: 6, // 3次迭代
                // memory_cost: 65536, // 64MB (单位是KB)
                memory_cost: 131072, // 128MB (单位是KB)
                parallelism: 8,
                salt,
            },
        }
    }

    pub fn recommended_params_with_salt(salt: &[u8]) -> Self {
        Self {
            params: Argon2idParams {
                dklen: 32,    // 32字节密钥
                time_cost: 5, // 5次迭代
                // memory_cost: 65536, // 64MB (单位是KB)
                memory_cost: 131072, // 128MB (单位是KB)
                // memory_cost: 262144, // 256MB (单位是KB)
                parallelism: 8,
                salt: salt.to_vec(),
            },
        }
    }

    /// 根据目标时间和系统资源动态校准参数
    pub fn calibrate_params(
        salt: &[u8],
        max_memory_kb: u32,
        max_time_ms: u32,
        parallelism: u32,
    ) -> Result<Self, KeystoreError> {
        // 基础参数（根据论文最低要求）
        let mut params = Argon2idParams {
            dklen: 32,
            time_cost: 1,
            memory_cost: max_memory_kb.min(15360), // 最低15MB
            parallelism,
            salt: salt.to_vec(),
        };

        // 动态校准迭代次数
        let (actual_time, t_cost) =
            Self::find_optimal_t_cost(params.memory_cost, params.parallelism, max_time_ms)?;

        params.time_cost = t_cost;
        tracing::info!(
            "Calibrated: m={}KB, t={}, p={}, time={}ms",
            params.memory_cost,
            params.time_cost,
            params.parallelism,
            actual_time
        );

        Ok(Self { params })
    }

    /// 寻找满足时间限制的最大t_cost
    fn find_optimal_t_cost(
        m_cost: u32,
        p_cost: u32,
        max_time_ms: u32,
    ) -> Result<(u128, u32), KeystoreError> {
        let mut t_cost = 1;
        let mut best_time = 0;
        let test_pwd = b"calibration_password";

        // 测试基准耗时
        let (base_time, _) = Self::measure_execution(m_cost, p_cost, 1, test_pwd)?;
        tracing::info!("Base time: {}ms", base_time);
        if base_time > max_time_ms as u128 {
            return Err(KeystoreError::Argon2(argon2::Error::MemoryTooLittle));
        }

        // 指数增长寻找上限
        while let Ok((time, _)) = Self::measure_execution(m_cost, p_cost, t_cost * 2, test_pwd) {
            if time > max_time_ms as u128 {
                break;
            }
            t_cost *= 2;
        }

        // 二分查找确定最优值
        let (mut low, mut high) = (t_cost, t_cost * 2);
        while low <= high {
            let mid = (low + high) / 2;
            match Self::measure_execution(m_cost, p_cost, mid, test_pwd) {
                Ok((time, _)) if time <= max_time_ms as u128 => {
                    best_time = time;
                    t_cost = mid;
                    low = mid + 1;
                }
                _ => high = mid - 1,
            }
        }

        Ok((best_time, t_cost))
    }

    /// 测量单次执行时间
    fn measure_execution(
        m_cost: u32,
        p_cost: u32,
        t_cost: u32,
        pwd: &[u8],
    ) -> Result<(u128, Vec<u8>), KeystoreError> {
        let params = Argon2idParams {
            dklen: 32,
            time_cost: t_cost,
            memory_cost: m_cost,
            parallelism: p_cost,
            salt: vec![0u8; 16], // 测试用固定盐
        };

        let kdf = Argon2idKdf { params };
        let start = std::time::Instant::now();
        let key = kdf.derive_key(pwd)?;
        Ok((start.elapsed().as_millis(), key))
    }

    /// 创建符合论文建议的安全参数
    pub fn secure_params(
        salt: &[u8],
        target_time_ms: Option<u32>,
        max_memory_kb: Option<u32>,
    ) -> Result<Self, KeystoreError> {
        // 默认安全基准（论文第5-6步）
        let default_memory = 65536; // 64MB
        let default_parallelism = 4;
        let max_memory = max_memory_kb.unwrap_or(default_memory);

        // 自动校准或使用默认值
        if let Some(target_time) = target_time_ms {
            Self::calibrate_params(salt, max_memory, target_time, default_parallelism)
        } else {
            Ok(Self {
                params: Argon2idParams {
                    dklen: 32,
                    time_cost: 3,
                    memory_cost: default_memory,
                    parallelism: default_parallelism,
                    salt: salt.to_vec(),
                },
            })
        }
    }
}

impl KeyDerivationFunction for Argon2idKdf {
    fn derive_key(&self, password: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        let start = std::time::Instant::now();
        let params = argon2::Params::new(
            self.params.memory_cost,
            self.params.time_cost,
            self.params.parallelism,
            None,
        )
        .map_err(KeystoreError::Argon2)?;

        let argon2 =
            argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

        let mut output_key = vec![0u8; self.params.dklen as usize];
        argon2
            .hash_password_into(password, &self.params.salt, &mut output_key)
            .map_err(KeystoreError::Argon2)?;
        tracing::info!("Argon2id KDF took {}ms", start.elapsed().as_millis());
        Ok(output_key)
    }

    fn params(&self) -> KdfParams {
        KdfParams::Argon2id(self.params.clone())
    }

    fn algorithm(&self) -> KdfAlgorithm {
        KdfAlgorithm::Argon2id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn test_key_derivation() {
        let mut rng = OsRng;
        let kdf = Argon2idKdf::recommended_params(&mut rng);
        let password = b"my_strong_password";

        // 测试正常派生
        let key1 = kdf.derive_key(password).unwrap();
        assert_eq!(key1.len(), 32);

        // 测试相同密码生成相同密钥（相同盐）
        let key2 = kdf.derive_key(password).unwrap();
        assert_eq!(key1, key2);

        // 测试不同盐生成不同密钥
        let mut different_salt_params = kdf.params.clone();
        different_salt_params.salt.iter_mut().for_each(|b| *b = !*b);
        let kdf2 = Argon2idKdf::new(different_salt_params);
        let key3 = kdf2.derive_key(password).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_invalid_params() {
        let invalid_params = Argon2idParams {
            dklen: 32,
            time_cost: 0, // 无效参数
            memory_cost: 8,
            parallelism: 1,
            salt: vec![0; 16],
        };

        let kdf = Argon2idKdf::new(invalid_params);
        let result = kdf.derive_key(b"password");
        assert!(matches!(result, Err(KeystoreError::Argon2(_))));
    }

    #[test]
    fn test_dynamic_calibration() {
        wallet_utils::init_test_log();
        // 测试快速模式（100ms时间限制）
        let salt = "somesalt".as_bytes();
        let kdf = Argon2idKdf::calibrate_params(salt, 131072, 200, 2).unwrap();
        assert!(kdf.params.time_cost >= 2);
        assert!(kdf.params.memory_cost <= 65536);

        // 测试内存限制
        let result = Argon2idKdf::calibrate_params(salt, 65536, 1000, 1);
        assert!(matches!(result, Err(KeystoreError::Argon2(_))));
    }

    #[test]
    fn test_secure_params() {
        wallet_utils::init_test_log();
        let salt = "somesalt".as_bytes();
        let kdf = Argon2idKdf::secure_params(salt, Some(500), None).unwrap();

        // 验证参数在合理范围
        assert!(kdf.params.memory_cost >= 15360);
        assert!(kdf.params.time_cost >= 2);
        assert!(kdf.params.parallelism >= 1);
    }
}
