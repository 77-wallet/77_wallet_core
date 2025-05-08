use crate::types;

use super::provider::Provider;
use sui_sdk::rpc_types::SuiTransactionBlockEffectsAPI;

use sui_types::transaction::{SenderSignedData, TransactionDataAPI};
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
        T: types::Transaction<sui_types::transaction::TransactionData>,
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
        let mut tx_data: sui_types::transaction::TransactionData = params.build_transaction()?;

        let gas_price = self.provider.get_reference_gas_price().await?;

        // 2. 干跑获得实际 gas_used 并调整 gas_budget
        let dry_run_result = self.provider.dry_run_transaction(&tx_data).await?;
        let gas_used = dry_run_result.effects.gas_cost_summary().gas_used();
        let buffer = (gas_used as f64 * 0.2).ceil() as u64;

        // 根据 buffer 更新 tx_data 中的 gas_budget 字段
        tx_data.gas_data_mut().budget = gas_used + buffer;

        // 3. 使用 Envelope 进行签名
        use sui_types::crypto::Signer;

        let raw_bytes = wallet_utils::serde_func::bcs_to_bytes(&tx_data)?;
        let signature = keypair.sign(&raw_bytes);
        // 构造 Signed Transaction

        let sender = SenderSignedData::new_from_sender_signature(tx_data, signature);
        let signed_tx = sui_types::transaction::Transaction::new(sender);

        // 4. 序列化已签名信封并编码
        let signed_bytes: Vec<u8> = wallet_utils::serde_func::bcs_to_bytes(&signed_tx)?;
        let signed_b64 = wallet_utils::base58_encode(&signed_bytes);

        // 5. 提交
        let tx_hash = self.provider.send_transaction(signed_b64).await?;
        Ok(tx_hash)
    }
}

#[cfg(test)]
mod tests {
    use crate::sui::SuiChain;

    use super::*;
    use wallet_transport::client::RpcClient;
    use wallet_utils::init_test_log;

    // Sui DevNet 节点地址
    const DEVNET_URL: &str = "https://fullnode.devnet.sui.io:443";
    const TESTNET_URL: &str = "https://fullnode.testnet.sui.io:443";
    // 测试用地址（Sui DevNet 水龙头示例地址）
    const TEST_ADDRESS: &str = "0x885f29a4f1b4d63822728a1b1811d0278c4e25f27d3754ddd387cd34f9482d0f";
    const TEST_COIN_TYPE: &str = "0x2::sui::SUI";

    fn get_chain() -> SuiChain {
        init_test_log();
        // sui 测试网络
        let rpc = TESTNET_URL.to_string();

        let header = None;
        let client = RpcClient::new(&rpc, header, None).unwrap();
        let provider = Provider::new(client).unwrap();
        let chain_code = wallet_types::chain::chain::ChainCode::Sui;
        let network = wallet_types::chain::network::NetworkKind::Testnet;
        let sui = SuiChain::new(provider, network, chain_code).unwrap();

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

    // #[tokio::test]
    // async fn test_estimate_gas() {
    //     let sui = get_chain();
    //     let gas = sui.estimate_gas(()).await.unwrap();
    //     println!("gas: {}", gas);
    // }
}
