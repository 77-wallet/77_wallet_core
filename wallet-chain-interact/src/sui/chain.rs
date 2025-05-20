use crate::QueryTransactionResult;
use crate::types::{self, ChainPrivateKey};

use super::provider::Provider;
use alloy::primitives::U256;
use shared_crypto::intent::{Intent, IntentMessage};

use sui_json_rpc_types::{
    Coin, SuiTransactionBlockEffects, SuiTransactionBlockEffectsAPI as _,
    SuiTransactionBlockResponse,
};
use sui_types::crypto::{AccountKeyPair, AccountPrivateKey, Signature};
use sui_types::transaction::{TransactionData, TransactionDataAPI};
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
    pub async fn balance(&self, addr: &str, token: Option<String>) -> crate::Result<U256> {
        let res = if let Some(t) = token
            && !t.is_empty()
        {
            self.provider.token_balance(addr, &t).await
        } else {
            self.provider.balance(addr).await
        }?;

        let res = res.total_balance;
        Ok(U256::from(res))
    }

    pub async fn block_num(&self) -> crate::Result<u64> {
        let latest_block = self.provider.latest_block().await?;

        Ok(wallet_utils::parse_func::u64_from_str(&latest_block)?)
    }
    pub async fn query_tx_res(
        &self,
        digest: &str,
    ) -> crate::Result<Option<QueryTransactionResult>> {
        let tx = self.provider.query_tx_info(digest).await?;

        let transaction_time = tx.timestamp_ms.map(|c| c as u128).unwrap_or_default();
        let transaction_fee = Self::extract_gas_used(&tx).unwrap_or_default();
        let status = Self::extract_status(&tx);
        let block_height = tx.checkpoint.map(|c| c as u128).unwrap_or_default();
        QueryTransactionResult::new(
            digest.to_string(),
            transaction_fee,
            "gas".to_owned(),
            transaction_time,
            status,
            block_height,
        );
        todo!()
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
        private_key: ChainPrivateKey,
        // keypair: sui_types::crypto::AccountKeyPair,
    ) -> crate::Result<SuiTransactionBlockResponse>
    where
        T: crate::types::Transaction<TransactionData>,
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

        let pkey_bytes = private_key.to_bytes()?;
        use sui_types::crypto::ToFromBytes;

        let key = AccountPrivateKey::from_bytes(pkey_bytes.as_slice()).unwrap();

        let keypair = AccountKeyPair::from(key);
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

    pub fn extract_gas_used(resp: &SuiTransactionBlockResponse) -> Option<f64> {
        let effects = resp.effects.as_ref()?;
        let gas_summary = match effects {
            SuiTransactionBlockEffects::V1(v1) => &v1.gas_used,
        };
        let mist = gas_summary.net_gas_usage();
        let sui = wallet_utils::unit::mist_to_sui(mist);
        Some(sui)
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

    async fn fetch_sorted(&self, owner: &str, coin_type: &str) -> crate::Result<Vec<Coin>> {
        let mut coins = self
            .provider
            .get_all_coins_by_owner(owner, coin_type)
            .await?;
        coins.sort_by_key(|c| std::cmp::Reverse(c.balance));
        Ok(coins)
    }

    pub async fn select_sufficient_coins(
        &self,
        owner: &str,
        coin_type: Option<&str>,
        amount_needed: u64,
    ) -> crate::Result<(
        Vec<sui_types::base_types::ObjectRef>,
        Vec<sui_types::base_types::ObjectRef>,
    )> {
        // 1. Select transfer coins (custom token or SUI)
        let transfer_type = coin_type.unwrap_or("0x2::sui::SUI");
        let coins = self.fetch_sorted(owner, transfer_type).await?;

        let mut transfer_coins = Vec::new();
        let mut sum = 0u128;
        for coin in &coins {
            let obj = (coin.coin_object_id, coin.version, coin.digest);
            if sum < amount_needed as u128 {
                transfer_coins.push(obj);
                sum += coin.balance as u128;
            } else {
                break;
            }
        }
        if sum < amount_needed as u128 {
            return Err(crate::Error::SuiError(
                crate::sui::error::SuiError::InsufficientBalance(sum as u64, amount_needed),
            ));
        }

        // 2. Select gas coins
        let gas_coins = if coin_type.is_none() {
            // For SUI transfers, remaining coins from the same list
            coins
                .into_iter()
                .skip(transfer_coins.len())
                .map(|c| (c.coin_object_id, c.version, c.digest))
                .collect()
        } else {
            // For custom token transfer, fetch SUI coins for gas
            let sui_coins = self.fetch_sorted(owner, "0x2::sui::SUI").await?;
            sui_coins
                .into_iter()
                .map(|c| (c.coin_object_id, c.version, c.digest))
                .collect()
        };
        Ok((transfer_coins, gas_coins))
    }
}

#[cfg(test)]
mod tests {
    use crate::sui::{SuiChain, TransferOpt};

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
        let provider = Provider::new(client);
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

        let pkey = "eae2c009537e02f1c1dff4859065dbaeefa098abe730a4fcb52c52704b781456";
        // let pkey_bytes = hex::decode(pkey).unwrap();
        // let key = AccountPrivateKey::from_bytes(pkey_bytes.as_slice()).unwrap();

        // let keypair = AccountKeyPair::from(key);
        let to = "0x807718c3c1f0cadc2c5715fb1d42fb4714e9a6b43c1df68b8b9c3773ccd93545";

        let amount = 1;
        let (transfer_coins, gas_coins) = sui
            .select_sufficient_coins(TEST_ADDRESS, None, amount)
            .await
            .unwrap();

        // let gas_budget = 1;
        // let gas_price = sui.provider.get_reference_gas_price().await.unwrap();
        // let contract = None;

        let params =
            TransferOpt::new(TEST_ADDRESS, to, amount, transfer_coins, gas_coins, None).unwrap();
        let gas = sui.exec_transaction(params, pkey.into()).await.unwrap();
        println!("gas: {}", gas);
    }

    #[tokio::test]
    async fn test_execute_token_transaction() {
        let sui = get_chain();

        let pkey = "eae2c009537e02f1c1dff4859065dbaeefa098abe730a4fcb52c52704b781456";
        // let pkey_bytes = hex::decode(pkey).unwrap();
        // let key = AccountPrivateKey::from_bytes(pkey_bytes.as_slice()).unwrap();

        // let keypair = AccountKeyPair::from(key);
        let to = "0x807718c3c1f0cadc2c5715fb1d42fb4714e9a6b43c1df68b8b9c3773ccd93545";
        let to = "0xa042c3ba8208964374cc050922ec94e85fdffe9fc0cd656fb623642ae2fdb4c0";

        let amount = 20;
        let contract =
            "0x1b9e65276fbeab5569a0afb074bb090b9eb867082417b0470a1a04f4be6d2f3a::qtoken::QTOKEN";
        let (transfer_coins, gas_coins) = sui
            .select_sufficient_coins(TEST_ADDRESS, Some(contract), amount)
            .await
            .unwrap();
        let id = transfer_coins[0].0;
        let obj = sui.provider.get_object_by_id(&id.to_string()).await;
        tracing::info!("obj: {:?}", obj);
        // let gas_budget = 1;
        // let gas_price = sui.provider.get_reference_gas_price().await.unwrap();
        // let contract = None;

        let params = TransferOpt::new(
            TEST_ADDRESS,
            to,
            amount,
            transfer_coins,
            gas_coins,
            Some(contract.to_string()),
        )
        .unwrap();
        let gas = sui.exec_transaction(params, pkey.into()).await;
        println!("gas: {:?}", gas);
    }
}
