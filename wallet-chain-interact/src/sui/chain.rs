use crate::types;

use super::provider::Provider;
use sui_sdk::rpc_types::SuiTransactionBlockEffectsAPI;
use sui_types::crypto::EmptySignInfo;
use sui_types::message_envelope::Envelope;
use wallet_types::chain::chain::ChainCode;
use wallet_types::chain::network;

pub struct SuiChain {
    pub provider: Provider,
    chain_code: ChainCode,
    network: network::NetworkKind,
}

impl SuiChain {
    pub fn new(
        provider: Provider,
        network: network::NetworkKind,
        chain_code: ChainCode,
    ) -> crate::Result<Self> {
        Ok(Self {
            provider,
            chain_code,
            network,
        })
    }
}

impl SuiChain {
    pub async fn balance(
        &self,
        addr: &str,
        token: Option<String>,
    ) -> crate::Result<sui_sdk::rpc_types::Balance> {
        if let Some(t) = token
            && !t.is_empty()
        {
            self.provider.token_balance(addr, &t).await
        } else {
            self.provider.balance(addr).await
        }
    }

    pub async fn estimate_gas<T>(&self, params: T) -> crate::Result<u64>
    where
        T: types::Transaction<TransactionData>,
    {
        let tx_data = params.build_transaction()?;
        self.provider
            .dry_run_transaction(&tx_data)
            .await
            .map(|res| res.effects.gas_cost_summary().gas_used())
    }
}

impl SuiChain {
    pub async fn exec_transaction<T>(
        &self,
        params: T,
        keypair: sui_types::crypto::AccountKeyPair,
    ) -> crate::Result<String>
    where
        T: crate::types::Transaction<sui_sdk::types::transaction::TransactionData>,
    {
        // 1. 构建原始 TransactionData
        let tx_data: sui_types::transaction::TransactionData = params.build_transaction()?;

        // 2. 干跑获得实际 gas_used 并调整 gas_budget
        let dry_run_result = self.provider.dry_run_transaction(&tx_data).await?;
        let gas_used = dry_run_result.effects.gas_cost_summary().gas_used();
        let buffer = (gas_used as f64 * 0.2).ceil() as u64;

        // 根据 buffer 更新 tx_data 中的 gas_budget 字段
        // tx_data = tx_data
        // ..with_gas_budget(gas_used + buffer);

        // 3. 使用 Envelope 进行签名
        use sui_types::crypto::KeypairTraits;
        use sui_types::crypto::Signer;

        let raw_bytes = wallet_utils::serde_func::bcs_to_bytes(&tx_data)?;
        let signature = keypair.sign(&raw_bytes);
        // 构造 Signed Transaction

        let signed_tx =
            sui_types::transaction::Transaction::new_from_data_and_sig(tx_data, signature);

        // 4. 序列化已签名信封并编码
        let signed_bytes: Vec<u8> = wallet_utils::serde_func::bcs_to_bytes(&signed_tx)?;
        let signed_b64 = wallet_utils::base58_encode(&signed_bytes);

        // 5. 提交
        let tx_hash = self.provider.send_transaction(signed_b64).await?;
        Ok(tx_hash)
    }
}
