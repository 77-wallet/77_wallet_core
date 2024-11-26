#[derive(Debug)]
pub struct MultiSigAccountResp {
    pub addr: String,
    pub tx_hash: String,
}

// 创建多签交易的响应数据
#[derive(Debug)]
pub struct MultisigTxResp {
    pub tx_hash: String,
    // 交易的原始数据
    pub raw_data: String,
}

#[derive(Debug)]
pub struct MultisigSignResp {
    //  签名的消息
    pub tx_hash: String,
    // 签名
    pub signature: String,
}
impl MultisigSignResp {
    pub fn new(signature: String) -> Self {
        Self {
            tx_hash: "".to_string(),
            signature,
        }
    }
    pub fn new_with_tx_hash(tx_hash: String, signature: String) -> Self {
        Self { tx_hash, signature }
    }
}

#[derive(Debug)]
pub struct FetchMultisigAddressResp {
    pub authority_address: String,
    pub multisig_address: String,
    pub salt: String,
}
impl FetchMultisigAddressResp {
    pub fn new_with_salt(multisig_address: String, salt: String) -> Self {
        Self {
            authority_address: "".to_string(),
            multisig_address,
            salt,
        }
    }
}
