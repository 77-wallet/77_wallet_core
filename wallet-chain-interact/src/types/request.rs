use wallet_types::valueobject::AddressPubkey;

#[derive(Clone)]
pub struct MultiSigAccount {
    pub from: String,
    pub token: Option<String>,
    pub threshold: u8,
    pub owners: Vec<AddressPubkey>,
    pub key: String,
    /// For Tron, the salt is equal to the address;
    /// for Solana, the salt is equal to a temporary public key;
    /// for Ethereum-based chains, the salt is equal to a random number;
    /// for BTC, the salt is equal to the script
    pub salt: String,
    pub address_type: String,
}
impl MultiSigAccount {
    pub fn new(from: String, threshold: u8, owners: Vec<AddressPubkey>) -> Self {
        Self {
            from,
            token: None,
            threshold,
            owners,
            key: "".to_string(),
            salt: "".to_string(),
            address_type: "".to_string(),
        }
    }

    pub fn with_key(mut self, key: String) -> Self {
        self.key = key;
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn with_nonce(mut self, nonce: String) -> Self {
        self.salt = nonce;
        self
    }

    pub fn with_address_type(mut self, address_type: String) -> Self {
        self.address_type = address_type;
        self
    }
}

// 执行多签交易的参数
pub struct ExecTransactionParams {
    pub executor: Option<Executor>,
    pub raw_data: String,
    pub seq_sign: Vec<String>,
    pub script_hex: String,
    pub address_type: String,
    pub internal_pubkey: String,
}

pub struct Executor {
    // transaction executor
    pub executor: String,
    // destination address(usually a contract address)
    pub destination: String,
    pub private_key: String,
    pub fee_setting: String,
}

impl Executor {
    pub fn new(
        executor: String,
        destination: String,
        private_key: String,
        fee_setting: String,
    ) -> Self {
        Self {
            executor,
            destination,
            private_key,
            fee_setting,
        }
    }
}
impl ExecTransactionParams {
    pub fn new(data: String, seq_sign: Vec<String>) -> Self {
        Self {
            executor: None,
            raw_data: data,
            seq_sign,
            script_hex: "".to_string(),
            address_type: "".to_string(),
            internal_pubkey: "".to_string(),
        }
    }
    pub fn with_executor(mut self, executor: Executor) -> Self {
        self.executor = Some(executor);
        self
    }
    pub fn with_address_type(mut self, address_type: &str) -> Self {
        self.address_type = address_type.to_string();
        self
    }
    pub fn with_internal_pubkey(mut self, internal_pubkey: &str) -> Self {
        self.internal_pubkey = internal_pubkey.to_string();
        self
    }
    pub fn with_script_hex(mut self, script_hex: &str) -> Self {
        self.script_hex = script_hex.to_string();
        self
    }
}

// pub enum BtcAddressType {}

pub struct SignTransactionArgs {
    pub from: Option<String>,
    pub key: String,
    pub raw_data_str: String,
    pub address_type: Option<String>,
    pub salt: Option<String>,
}
impl SignTransactionArgs {
    pub fn new(from: &str, key: &str, raw_data: &str, address: &str, salt: &str) -> Self {
        let address_type = if address.is_empty() {
            None
        } else {
            Some(address.to_string())
        };

        let salt = if salt.is_empty() {
            None
        } else {
            Some(salt.to_string())
        };

        Self {
            from: Some(from.to_string()),
            key: key.to_string(),
            raw_data_str: raw_data.to_string(),
            address_type,
            salt,
        }
    }
}

// TODO 逐步优化,考虑使用u8字节
#[derive(Debug)]
pub struct ChainPrivateKey(String);

impl std::ops::Deref for ChainPrivateKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for ChainPrivateKey {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for ChainPrivateKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}
