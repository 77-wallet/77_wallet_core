use super::TransRespOpt;
use super::consts::{self, DEFAULT_GAS_BUDGET, SUI_VALUE};
use super::error::SuiError;
use super::protocol::EstimateFeeResp;
use super::provider::Provider;
use crate::types::ChainPrivateKey;
use crate::{BillResourceConsume, QueryTransactionResult};
use alloy::primitives::U256;
use shared_crypto::intent::{Intent, IntentMessage};
use sui_json_rpc_types::{
    SuiTransactionBlockDataAPI, SuiTransactionBlockEffects, SuiTransactionBlockEffectsAPI as _,
    SuiTransactionBlockResponse,
};
use sui_types::crypto::{AccountKeyPair, AccountPrivateKey, Signature};
use sui_types::transaction::{ProgrammableTransaction, TransactionData};

pub struct SuiChain {
    pub provider: Provider,
}

impl SuiChain {
    pub fn new(provider: Provider) -> crate::Result<Self> {
        Ok(Self { provider })
    }
}

impl SuiChain {
    pub async fn balance(&self, addr: &str, token: Option<String>) -> crate::Result<U256> {
        let coin_type = if let Some(t) = &token {
            t
        } else {
            consts::SUI_NATIVE_COIN
        };

        let res = self.provider.balance(addr, &coin_type).await?;

        Ok(U256::from(res.total_balance))
    }

    pub async fn block_num(&self) -> crate::Result<u64> {
        let latest_block = self.provider.latest_block().await?;

        Ok(wallet_utils::parse_func::u64_from_str(&latest_block)?)
    }
    pub async fn query_tx_res(
        &self,
        digest: &str,
    ) -> crate::Result<Option<QueryTransactionResult>> {
        let opt = TransRespOpt::default();
        let tx = self.provider.query_tx_info(digest, opt).await?;

        let transaction_time = tx.timestamp_ms.map(|c| c as u128).unwrap_or_default();
        let transaction_fee = Self::extract_gas_used(&tx).unwrap_or_default();
        let status = Self::extract_status(&tx);
        let block_height = tx.checkpoint.map(|c| c as u128).unwrap_or_default();

        let resource_consume =
            BillResourceConsume::one_resource(transaction_fee.1 as u64).to_json_str()?;

        let result = QueryTransactionResult::new(
            digest.to_string(),
            transaction_fee.0,
            resource_consume,
            transaction_time,
            status,
            block_height,
        );

        Ok(Some(result))
    }
    pub async fn decimals(&self, token_addr: &str) -> crate::Result<u8> {
        let meta_data = self.provider.get_coin_metadata(token_addr).await?;

        Ok(meta_data.decimals)
    }
    pub async fn token_symbol(&self, token: &str) -> crate::Result<String> {
        let meta_data = self.provider.get_coin_metadata(token).await?;
        Ok(meta_data.symbol)
    }
    pub async fn token_name(&self, token: &str) -> crate::Result<String> {
        let meta_data = self.provider.get_coin_metadata(token).await?;
        Ok(meta_data.name)
    }

    pub async fn estimate_fee(
        &self,
        sender: &str,
        tx: ProgrammableTransaction,
    ) -> crate::Result<EstimateFeeResp> {
        let gas_price = self.provider.get_reference_gas_price().await?;

        let result = self
            .provider
            .dev_inspect_transaction(sender, tx, gas_price)
            .await?;

        if result.effects.status().is_err() {
            return Err(SuiError::GasError(result.effects.status().to_string()))?;
        }

        let gas_used = result.effects.gas_cost_summary().computation_cost
            + result.effects.gas_cost_summary().storage_cost;
        let gas_used = if gas_used <= 0 {
            DEFAULT_GAS_BUDGET
        } else {
            gas_used as u64
        };

        Ok(EstimateFeeResp::new(gas_used, gas_price))
    }

    pub async fn exec(
        &self,
        tx_data: TransactionData,
        private_key: ChainPrivateKey,
    ) -> crate::Result<String> {
        let tx_bytes = wallet_utils::serde_func::bcs_to_bytes(&tx_data)?;

        let intent_msg = IntentMessage::new(Intent::sui_transaction(), tx_data);
        // 用 keypair 对 IntentMessage 进行签名

        let pubkey_bytes = private_key.to_bytes()?;
        use sui_types::crypto::ToFromBytes;

        let key = AccountPrivateKey::from_bytes(pubkey_bytes.as_slice()).unwrap();

        let keypair = AccountKeyPair::from(key);
        let signature = Signature::new_secure(&intent_msg, &keypair);

        let tx_data_base64 = wallet_utils::bytes_to_base64(&tx_bytes);
        let sig_b64 = wallet_utils::bytes_to_base64(signature.as_ref());

        // 5. 提交
        let tx_hash = self
            .provider
            .send_transaction(tx_data_base64, vec![sig_b64])
            .await?;
        Ok(tx_hash.digest.to_string())
    }
}

impl SuiChain {
    pub fn extract_gas_used(resp: &SuiTransactionBlockResponse) -> Option<(f64, i64)> {
        let effects = resp.effects.as_ref()?;

        let gas_price = resp
            .transaction
            .as_ref()
            .map(|t| t.data.gas_data().price)
            .unwrap_or_default();
        let gas_summary = match effects {
            SuiTransactionBlockEffects::V1(v1) => &v1.gas_used,
        };
        let fee = gas_summary.net_gas_usage();

        let gas_used = fee / gas_price as i64;

        Some((fee as f64 / SUI_VALUE, gas_used))
    }

    pub fn extract_status(resp: &SuiTransactionBlockResponse) -> i8 {
        let Some(effects) = resp.effects.as_ref() else {
            return 3;
        };
        let status = match effects {
            SuiTransactionBlockEffects::V1(v1) => &v1.status,
        };
        match status {
            sui_json_rpc_types::SuiExecutionStatus::Success => 2,
            sui_json_rpc_types::SuiExecutionStatus::Failure { error: _ } => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sui::{SuiChain, transfer::TransferOpt};
    use wallet_transport::client::RpcClient;
    use wallet_utils::init_test_log;

    const TESTNET_URL: &str = "https://fullnode.testnet.sui.io:443";
    const TEST_ADDRESS: &str = "0x885f29a4f1b4d63822728a1b1811d0278c4e25f27d3754ddd387cd34f9482d0f";
    const TEST_COIN_TYPE: &str = "0x2::sui::SUI";

    fn get_chain() -> SuiChain {
        init_test_log();
        // sui 测试网络
        let rpc = TESTNET_URL.to_string();

        let header = None;
        let client = RpcClient::new(&rpc, header, None).unwrap();
        let provider = Provider::new(client);

        let sui = SuiChain::new(provider).unwrap();
        sui
    }

    #[tokio::test]
    async fn test_balance() {
        let sui = get_chain();

        let balance = sui
            .balance(TEST_ADDRESS, Some(TEST_COIN_TYPE.to_string()))
            .await
            .unwrap();
        println!("{:?}", balance);
    }

    #[tokio::test]
    async fn test_token_balance() {
        let sui = get_chain();

        let contract =
            "0x1b9e65276fbeab5569a0afb074bb090b9eb867082417b0470a1a04f4be6d2f3a::qtoken::QTOKEN";

        let balance = sui
            .balance(TEST_ADDRESS, Some(contract.to_string()))
            .await
            .unwrap();
        println!("{:?}", balance);
    }

    #[tokio::test]
    async fn test_execute_transaction() {
        let sui = get_chain();

        let pubkey = "eae2c009537e02f1c1dff4859065dbaeefa098abe730a4fcb52c52704b781456";

        let to = "0xa042c3ba8208964374cc050922ec94e85fdffe9fc0cd656fb623642ae2fdb4c0";
        let value = wallet_utils::unit::convert_to_u256("0.2", 9).unwrap();
        let token = None;

        let params = TransferOpt::new(TEST_ADDRESS, to, value, token).unwrap();

        let mut helper = params.select_coin(&sui.provider).await.unwrap();

        // 预估手续费
        let pt = params
            .build_pt(&sui.provider, &mut helper, None)
            .await
            .unwrap();
        let gas_fee = sui.estimate_fee(TEST_ADDRESS, pt).await.unwrap();

        tracing::info!("fee {}", gas_fee.get_fee_f64());

        let trans = params
            .build_data(&sui.provider, helper, gas_fee)
            .await
            .unwrap();

        let hash = sui
            .exec(trans, ChainPrivateKey::from(pubkey))
            .await
            .unwrap();

        tracing::info!("hash: {}", hash);
    }

    #[tokio::test]
    async fn test_execute_token_transaction() {
        let sui = get_chain();

        let pubkey = "eae2c009537e02f1c1dff4859065dbaeefa098abe730a4fcb52c52704b781456";

        let to = "0x4c1cd48f7f203870be350d7a18c5a827131cecc7322b1571b9a69aeae7dda5f2";
        let value = wallet_utils::unit::convert_to_u256("0.1", 6).unwrap();
        let token = Some(
            "0x1b9e65276fbeab5569a0afb074bb090b9eb867082417b0470a1a04f4be6d2f3a::qtoken::QTOKEN"
                .to_string(),
        );

        let params = TransferOpt::new(TEST_ADDRESS, to, value, token).unwrap();
        let mut helper = params.select_coin(&sui.provider).await.unwrap();

        // 预估手续费
        let pt = params
            .build_pt(&sui.provider, &mut helper, None)
            .await
            .unwrap();
        let gas_fee = sui.estimate_fee(TEST_ADDRESS, pt).await.unwrap();

        let trans = params
            .build_data(&sui.provider, helper, gas_fee)
            .await
            .unwrap();

        let hash = sui
            .exec(trans, ChainPrivateKey::from(pubkey))
            .await
            .unwrap();

        tracing::info!("hash: {}", hash);
    }
}
