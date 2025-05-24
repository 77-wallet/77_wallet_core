use super::TransRespOpt;
use super::consts::{self, SUI_VALUE};
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

        let gas_used = result.effects.gas_cost_summary().net_gas_usage() as u64;

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

    // pub async fn estimate_gas<T>(&self, params: T) -> crate::Result<i64>
    // where
    //     T: types::Transaction<sui_types::transaction::TransactionData>,
    // {
    //     let tx_data = params.build_transaction()?;
    //     self.provider
    //         .dry_run_transaction(&tx_data)
    //         .await
    //         .map(|res| res.effects.gas_cost_summary().net_gas_usage())
    // }

    // async fn fetch_sorted(&self, owner: &str, coin_type: &str) -> crate::Result<Vec<Coin>> {
    //     let mut coins = self
    //         .provider
    //         .get_all_coins_by_owner(owner, coin_type)
    //         .await?;
    //     coins.sort_by_key(|c| std::cmp::Reverse(c.balance));
    //     Ok(coins)
    // }

    // pub async fn select_sufficient_coins(
    //     &self,
    //     owner: &str,
    //     coin_type: Option<&str>,
    //     amount_needed: u64,
    // ) -> crate::Result<(
    //     Vec<sui_types::base_types::ObjectRef>,
    //     Vec<sui_types::base_types::ObjectRef>,
    // )> {
    //     // 1. Select transfer coins (custom token or SUI)
    //     let transfer_type = coin_type.unwrap_or("0x2::sui::SUI");
    //     let coins = self.fetch_sorted(owner, transfer_type).await?;

    //     let mut transfer_coins = Vec::new();
    //     let mut sum = 0u128;
    //     for coin in &coins {
    //         let obj = (coin.coin_object_id, coin.version, coin.digest);
    //         if sum < amount_needed as u128 {
    //             transfer_coins.push(obj);
    //             sum += coin.balance as u128;
    //         } else {
    //             break;
    //         }
    //     }
    //     if sum < amount_needed as u128 {
    //         return Err(crate::Error::SuiError(
    //             crate::sui::error::SuiError::InsufficientBalance(sum as u64, amount_needed),
    //         ));
    //     }

    //     // 2. Select gas coins
    //     let gas_coins = if coin_type.is_none() {
    //         // For SUI transfers, remaining coins from the same list
    //         coins
    //             .into_iter()
    //             .skip(transfer_coins.len())
    //             .map(|c| (c.coin_object_id, c.version, c.digest))
    //             .collect()
    //     } else {
    //         // For custom token transfer, fetch SUI coins for gas
    //         let sui_coins = self.fetch_sorted(owner, "0x2::sui::SUI").await?;
    //         sui_coins
    //             .into_iter()
    //             .map(|c| (c.coin_object_id, c.version, c.digest))
    //             .collect()
    //     };
    //     Ok((transfer_coins, gas_coins))
    // }
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

        let to = "0xfba1550112b16f3608669c8ab4268366c7bacb3a2cb844594ad67c21af85a1dd";
        let value = wallet_utils::unit::convert_to_u256("1", 9).unwrap();
        let token = None;

        let params = TransferOpt::new(TEST_ADDRESS, to, value, token).unwrap();

        // 预估手续费
        let (pt, helper) = params.build_pt(&sui.provider).await.unwrap();
        let gas_fee = sui.estimate_fee(TEST_ADDRESS, pt).await.unwrap();

        let trans = params
            .build_data(&sui.provider, helper, gas_fee)
            .await
            .unwrap();

        let hash = sui
            .exec(trans, ChainPrivateKey::from(pubkey))
            .await
            .unwrap();

        println!("hash: {}", hash);
    }

    #[tokio::test]
    async fn test_execute_token_transaction() {
        let sui = get_chain();

        let pubkey = "eae2c009537e02f1c1dff4859065dbaeefa098abe730a4fcb52c52704b781456";

        let to = "0xfba1550112b16f3608669c8ab4268366c7bacb3a2cb844594ad67c21af85a1dd";
        let value = wallet_utils::unit::convert_to_u256("1", 9).unwrap();
        let token = Some(
            "0x1b9e65276fbeab5569a0afb074bb090b9eb867082417b0470a1a04f4be6d2f3a::qtoken::QTOKEN"
                .to_string(),
        );

        let params = TransferOpt::new(TEST_ADDRESS, to, value, token).unwrap();

        // 预估手续费
        let (pt, helper) = params.build_pt(&sui.provider).await.unwrap();
        let gas_fee = sui.estimate_fee(TEST_ADDRESS, pt).await.unwrap();

        let trans = params
            .build_data(&sui.provider, helper, gas_fee)
            .await
            .unwrap();

        let hash = sui
            .exec(trans, ChainPrivateKey::from(pubkey))
            .await
            .unwrap();

        println!("hash: {}", hash);
    }
}

// pub async fn exec_transaction<T>(
//     &self,
//     params: T,
//     private_key: ChainPrivateKey,
//     // keypair: sui_types::crypto::AccountKeyPair,
// ) -> crate::Result<SuiTransactionBlockResponse>
// where
//     T: crate::types::Transaction<TransactionData>,
// {
//     // 1. 构建原始 TransactionData
//     let mut tx_data: sui_types::transaction::TransactionData = params.build_transaction()?;
//     let gas_price = self.provider.get_reference_gas_price().await?;
//     tracing::info!("gas_price: {}", gas_price);
//     // 2. 干跑获得实际 gas_used 并调整 gas_budget
//     let dry_run_result = self.provider.dry_run_transaction(&tx_data).await?;
//     let gas_used = dry_run_result.effects.gas_cost_summary().gas_used();
//     let buffer = (gas_used as f64 * 0.2).ceil() as u64;

//     // 根据 buffer 更新 tx_data 中的 gas_budget 字段
//     let gas_data = tx_data.gas_data_mut();
//     gas_data.budget = gas_used + buffer;
//     gas_data.price = gas_price;
//     tracing::info!("gas_budget: {}", gas_data.budget);
//     tracing::info!("gas_price: {}", gas_data.price);
//     // let coins = self
//     //     .select_sufficient_coins(&tx_data.sender().to_string(), "0x2::sui::SUI")
//     //     .await?;

//     // 3. 使用 Envelope 进行签名
//     // let tx_data_bytes = wallet_utils::serde_func::bcs_to_bytes(&tx_data)?;

//     // let intent_message = IntentMessage::new(
//     //     Intent::sui_transaction(),
//     //     tx_data.clone(),
//     // );
//     // let signature = keypair.sign(&tx_data_bytes);
//     // 构造 Signed Transaction

//     // let sender = SenderSignedData::new_from_sender_signature(tx_data, signature);
//     // let signed_tx = sui_types::transaction::Transaction::new(sender);
//     let tx_bytes = wallet_utils::serde_func::bcs_to_bytes(&tx_data)?;

//     let intent_msg = IntentMessage::new(Intent::sui_transaction(), tx_data);
//     // 用 keypair 对 IntentMessage 进行签名

//     let pkey_bytes = private_key.to_bytes()?;
//     use sui_types::crypto::ToFromBytes;

//     let key = AccountPrivateKey::from_bytes(pkey_bytes.as_slice()).unwrap();

//     let keypair = AccountKeyPair::from(key);
//     let signature = Signature::new_secure(&intent_msg, &keypair);

//     let tx_data_base64 = wallet_utils::bytes_to_base64(&tx_bytes);
//     let sig_b64 = wallet_utils::bytes_to_base64(signature.as_ref());
//     // todo!();
//     // 4. 序列化已签名信封并编码
//     // let signed_bytes: Vec<u8> = wallet_utils::serde_func::bcs_to_bytes(&signed_tx)?;
//     // let signed_b64 = wallet_utils::bytes_to_base64(&signed_bytes);

//     // 5. 提交
//     let tx_hash = self
//         .provider
//         .send_transaction(tx_data_base64, vec![sig_b64])
//         .await?;
//     Ok(tx_hash)
// }
