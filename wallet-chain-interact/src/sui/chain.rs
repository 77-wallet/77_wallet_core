use crate::types;

use super::provider::Provider;
use shared_crypto::intent::{Intent, IntentMessage};
use sui_sdk::rpc_types::SuiTransactionBlockEffectsAPI;

use sui_types::crypto::Signature;
use sui_types::transaction::{SenderSignedData, TransactionData, TransactionDataAPI};
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
    ) -> crate::Result<sui_sdk::rpc_types::SuiTransactionBlockResponse>
    where
        T: crate::types::Transaction<sui_sdk::types::transaction::TransactionData>,
    {
        // 1. 构建原始 TransactionData
        let mut tx_data: sui_types::transaction::TransactionData = params.build_transaction()?;
        let gas_price = self.provider.get_reference_gas_price().await?;
        tracing::info!("gas_price: {}", gas_price);
        // 2. 干跑获得实际 gas_used 并调整 gas_budget
        let dry_run_result = self.provider.dry_run_transaction(&tx_data).await?;
        let gas_used = dry_run_result.effects.gas_cost_summary().gas_used();
        let buffer = (gas_used as f64 * 0.2).ceil() as u64;

        // 根据 buffer 更新 tx_data 中的 gas_budget 字段
        let gas_data = tx_data.gas_data_mut();
        gas_data.budget = gas_used + buffer;
        gas_data.price = gas_price;
        tracing::info!("gas_budget: {}", gas_data.budget);
        tracing::info!("gas_price: {}", gas_data.price);
        // let coins = self
        //     .select_sufficient_coins(&tx_data.sender().to_string(), "0x2::sui::SUI")
        //     .await?;

        // 3. 使用 Envelope 进行签名
        use sui_types::crypto::Signer;

        // let tx_data_bytes = wallet_utils::serde_func::bcs_to_bytes(&tx_data)?;

        // let intent_message = IntentMessage::new(
        //     Intent::sui_transaction(),
        //     tx_data.clone(),
        // );
        // let signature = keypair.sign(&tx_data_bytes);
        // 构造 Signed Transaction

        // let sender = SenderSignedData::new_from_sender_signature(tx_data, signature);
        // let signed_tx = sui_types::transaction::Transaction::new(sender);
        let tx_bytes = wallet_utils::serde_func::bcs_to_bytes(&tx_data)?;

        let intent_msg = IntentMessage::new(Intent::sui_transaction(), tx_data);
        // 用 keypair 对 IntentMessage 进行签名
        let signature = Signature::new_secure(&intent_msg, &keypair);

        let tx_data_base64 = wallet_utils::bytes_to_base64(&tx_bytes);
        let sig_b64 = wallet_utils::bytes_to_base64(signature.as_ref());
        // todo!();
        // 4. 序列化已签名信封并编码
        // let signed_bytes: Vec<u8> = wallet_utils::serde_func::bcs_to_bytes(&signed_tx)?;
        // let signed_b64 = wallet_utils::bytes_to_base64(&signed_bytes);

        // 5. 提交
        let tx_hash = self
            .provider
            .send_transaction(tx_data_base64, vec![sig_b64])
            .await?;
        Ok(tx_hash)
    }

    pub async fn select_sufficient_coins(
        &self,
        owner: &str,
        coin_type: &str,
        amount_needed: u64,
    ) -> crate::Result<(
        Vec<sui_types::base_types::ObjectRef>,
        Vec<sui_types::base_types::ObjectRef>,
    )> {
        let mut coins = self
            .provider
            .get_all_coins_by_owner(owner, coin_type)
            .await?;

        // 按余额从大到小排序（贪心）
        coins.sort_by(|a, b| b.balance.cmp(&a.balance));

        let mut transfer_coins = Vec::new();
        let mut gas_coins = Vec::new();
        let mut total: u64 = 0;

        for coin in coins {
            let obj = (coin.coin_object_id, coin.version, coin.digest);
            if total < amount_needed {
                transfer_coins.push(obj.clone());
            } else {
                gas_coins.push(obj.clone());
            }
            total += coin.balance;
        }

        // 2. 剩余的 coins 用于 gas
        // gas_coins.extend_from_slice(&coins[transfer_coins.len()..]);

        if total < amount_needed {
            Err(crate::Error::SuiError(
                crate::sui::error::SuiError::InsufficientBalance(total, amount_needed),
            ))
        } else {
            Ok((transfer_coins, gas_coins))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sui::{SuiChain, TransferOpt};

    use super::*;
    use sui_types::crypto::{AccountKeyPair, AccountPrivateKey, ToFromBytes};
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

    #[tokio::test]
    async fn test_execute_transaction() {
        let sui = get_chain();

        let pkey = "eae2c009537e02f1c1dff4859065dbaeefa098abe730a4fcb52c52704b781456";
        let pkey_bytes = hex::decode(pkey).unwrap();
        let key = AccountPrivateKey::from_bytes(pkey_bytes.as_slice()).unwrap();

        let keypair = AccountKeyPair::from(key);
        let to = "0x807718c3c1f0cadc2c5715fb1d42fb4714e9a6b43c1df68b8b9c3773ccd93545";

        let amount = 1;
        let (transfer_coins, gas_coins) = sui
            .select_sufficient_coins(TEST_ADDRESS, "0x2::sui::SUI", amount)
            .await
            .unwrap();
        // let gas_budget = 1;
        // let gas_price = sui.provider.get_reference_gas_price().await.unwrap();
        // let contract = None;

        let params =
            TransferOpt::new(TEST_ADDRESS, to, amount, transfer_coins, gas_coins, None).unwrap();
        let gas = sui.exec_transaction(params, keypair).await.unwrap();
        println!("gas: {}", gas);
    }
}
