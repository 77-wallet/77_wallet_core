use crate::eth::protocol::contract::{balanceOfCall, decimalsCall};
use alloy::{
    network::{eip2718::Encodable2718, TransactionBuilder},
    primitives::U256,
    rpc::types::{Block, TransactionInput, TransactionReceipt, TransactionRequest},
    sol_types::SolCall,
};
use serde_json::json;
use wallet_transport::{client::RpcClient, types::JsonRpcParams};
use wallet_utils::{address, unit};

use super::{
    protocol::contract::{isBlackListedCall, nameCall, symbolCall},
    EtherFee,
};

pub struct Provider {
    client: RpcClient,
}

impl Provider {
    pub fn new(rpc_client: RpcClient) -> crate::Result<Self> {
        Ok(Self { client: rpc_client })
    }

    pub async fn balance(&self, addr: &str) -> crate::Result<U256> {
        let params = JsonRpcParams::default()
            .method("eth_getBalance")
            .params(vec![addr, "latest"]);

        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;
        Ok(unit::u256_from_str(&r)?)
    }

    pub async fn is_contract_address(&self, addr: &str) -> crate::Result<String> {
        let params = JsonRpcParams::default()
            .method("eth_getCode")
            .params(vec![addr, "latest"]);

        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;
        Ok(r)
    }

    pub async fn estimate_gas(&self, tx: TransactionRequest) -> crate::Result<U256> {
        let params = JsonRpcParams::default()
            .method("eth_estimateGas")
            .params(vec![json!(tx)]);

        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;

        let gas = wallet_utils::unit::u256_from_str(&r)?;
        let ten_percent = (gas * U256::from(10)) / U256::from(100);
        Ok(gas + ten_percent)
    }

    // used testnet
    pub async fn get_fee(
        &self,
        tx: TransactionRequest,
    ) -> crate::Result<super::params::FeeSetting> {
        let gas_limit = self.estimate_gas(tx.clone()).await?;

        let max_priority_fee_per_gas = U256::from(2_000_000_000u64);
        let gas_price = self.gas_price().await?;
        let max_fee = gas_price + max_priority_fee_per_gas;
        let res = super::params::FeeSetting {
            base_fee: gas_price,
            gas_limit,
            max_fee_per_gas: max_fee,
            max_priority_fee_per_gas,
        };
        Ok(res)
    }

    pub async fn set_transaction_fee(
        &self,
        tx: TransactionRequest,
        fee: super::params::FeeSetting,
    ) -> crate::Result<TransactionRequest> {
        Ok(tx
            .with_gas_limit(fee.gas_limit.to::<u64>())
            .with_max_priority_fee_per_gas(fee.max_priority_fee_per_gas.to::<u128>())
            .with_max_fee_per_gas(fee.max_fee_per_gas.to::<u128>()))
    }

    pub async fn send_raw_transaction(
        &self,
        tx: TransactionRequest,
        key: &str,
    ) -> crate::Result<String> {
        let nonce = self.nonce(&tx.from.unwrap().to_string()).await?;
        let chain_id = self.chain_id().await?;
        let tx = tx.with_nonce(nonce).with_chain_id(chain_id);

        // 签名交易
        let signer: alloy::signers::local::PrivateKeySigner = key
            .parse()
            .map_err(|_| crate::Error::SignError("get singer from key error".to_string()))?;
        let wallet = alloy::network::EthereumWallet::from(signer);

        let tx_envelope = tx
            .build(&wallet)
            .await
            .map_err(|e| crate::Error::SignError(e.to_string()))?;
        tx_envelope.encoded_2718();
        let tx_encoded = tx_envelope.encoded_2718();

        let hex_raw = format!("0x{}", hex::encode(tx_encoded));
        let params = JsonRpcParams::default()
            .method("eth_sendRawTransaction")
            .params(vec![hex_raw]);

        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;
        Ok(r)
    }

    pub async fn token_balance(&self, addr: &str, token: &str) -> crate::Result<U256> {
        let token_addr = address::parse_eth_address(token)?;
        let addr = address::parse_eth_address(addr)?;

        let call = balanceOfCall { owner: addr };
        let data = call.abi_encode();

        let input = TransactionInput {
            input: None,
            data: Some(data.into()),
        };
        let tx_req = TransactionRequest::default().to(token_addr).input(input);

        let params = JsonRpcParams::default()
            .method("eth_call")
            .params(vec![json!(tx_req), "latest".into()]);

        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;
        Ok(wallet_utils::unit::u256_from_str(&r)?)
    }

    pub async fn eth_call(&self, tx: TransactionRequest) -> crate::Result<String> {
        let params = JsonRpcParams::default()
            .method("eth_call")
            .params(vec![json!(tx), json!("latest")]);

        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;

        Ok(r)
    }

    pub async fn get_block_height(&self) -> crate::Result<String> {
        let params: JsonRpcParams<()> = JsonRpcParams::default()
            .method("eth_blockNumber")
            .no_params();
        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;
        Ok(r)
    }

    // 代币精度
    pub async fn decimals(&self, token: &str) -> crate::Result<U256> {
        let token_addr = address::parse_eth_address(token)?;

        let call = decimalsCall {};
        let data = call.abi_encode();

        let input = TransactionInput {
            input: None,
            data: Some(data.into()),
        };
        let tx_req = TransactionRequest::default().to(token_addr).input(input);

        let params = JsonRpcParams::default()
            .method("eth_call")
            .params(vec![json!(tx_req), json!("latest")]);

        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;
        Ok(unit::u256_from_str(&r)?)
    }

    pub async fn token_symbol(&self, token: &str) -> crate::Result<String> {
        let token_addr = address::parse_eth_address(token)?;

        let call = symbolCall {};
        let data = call.abi_encode();

        let input = TransactionInput {
            input: None,
            data: Some(data.into()),
        };
        let tx_req = TransactionRequest::default().to(token_addr).input(input);

        let params = JsonRpcParams::default()
            .method("eth_call")
            .params(vec![json!(tx_req), json!("latest")]);

        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;

        Ok(r)
    }

    pub async fn token_name(&self, token: &str) -> crate::Result<String> {
        let token_addr = address::parse_eth_address(token)?;

        let call = nameCall {};
        let data = call.abi_encode();

        let input = TransactionInput {
            input: None,
            data: Some(data.into()),
        };
        let tx_req = TransactionRequest::default().to(token_addr).input(input);

        let params = JsonRpcParams::default()
            .method("eth_call")
            .params(vec![json!(tx_req), json!("latest")]);

        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;

        Ok(r)
    }

    pub async fn black_address(&self, token: &str, owner: &str) -> crate::Result<String> {
        let token_addr = address::parse_eth_address(token)?;
        let owner_addr = address::parse_eth_address(owner)?;

        let call = isBlackListedCall { from: owner_addr };
        let data = call.abi_encode();

        let input = TransactionInput {
            input: None,
            data: Some(data.into()),
        };
        let tx_req = TransactionRequest::default().to(token_addr).input(input);

        let params = JsonRpcParams::default()
            .method("eth_call")
            .params(vec![json!(tx_req), json!("latest")]);

        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;

        Ok(r)
    }

    pub async fn gas_price(&self) -> crate::Result<U256> {
        let params = JsonRpcParams::<String>::default().method("eth_gasPrice");

        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;
        Ok(unit::u256_from_str(&r)?)
    }

    pub async fn latest_block(&self) -> crate::Result<Block> {
        let hash = serde_json::to_value("latest").unwrap();
        let bool = serde_json::to_value(false).unwrap();

        let params = JsonRpcParams::default()
            .method("eth_getBlockByNumber")
            .params(vec![hash, bool]);

        let res = self
            .client
            .set_params(params)
            .send_json_rpc::<Block>()
            .await?;

        Ok(res)
    }

    // price unit is wei
    pub async fn get_default_fee(&self) -> crate::Result<EtherFee> {
        let block = self.latest_block().await?;

        let base_fee = match block.header.base_fee_per_gas {
            Some(base_fee) => {
                if base_fee > 0 {
                    U256::from(base_fee)
                } else {
                    U256::ZERO
                }
            }
            None => U256::ZERO,
        };

        let priority_fee_per_gas = self.gas_price().await? - base_fee;
        Ok(EtherFee {
            base_fee,
            priority_fee_per_gas,
        })
    }

    pub async fn base_fee(&self) -> crate::Result<U256> {
        let block = self.latest_block().await?;

        match block.header.base_fee_per_gas {
            Some(base_fee) => {
                if base_fee > 0 {
                    Ok(U256::from(base_fee))
                } else {
                    self.gas_price().await
                }
            }
            None => self.gas_price().await,
        }
    }

    pub async fn nonce(&self, addr: &str) -> crate::Result<u64> {
        let params = JsonRpcParams::default()
            .method("eth_getTransactionCount")
            .params(vec![addr, "latest"]);

        let rs = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;

        let chain_id = wallet_utils::unit::u256_from_str(&rs)?;
        Ok(chain_id.to::<u64>())
    }

    pub async fn chain_id(&self) -> crate::Result<u64> {
        let c: Vec<String> = Vec::with_capacity(1);
        let params = JsonRpcParams::default().method("eth_chainId").params(c);

        let rs = self
            .client
            .set_params(params)
            .send_json_rpc::<String>()
            .await?;

        let chain_id = wallet_utils::unit::u256_from_str(&rs)?;
        Ok(chain_id.to::<u64>())
    }

    // 查询交易的收据
    pub async fn transaction_receipt(&self, hash: &str) -> crate::Result<TransactionReceipt> {
        let params = JsonRpcParams::default()
            .method("eth_getTransactionReceipt")
            .params(vec![hash]);

        let rs = self
            .client
            .set_params(params)
            .send_json_rpc::<TransactionReceipt>()
            .await?;
        Ok(rs)
    }

    pub async fn block_by_hash(&self, hash: &str) -> crate::Result<Block> {
        let hash = serde_json::to_value(hash).unwrap();
        let bool = serde_json::to_value(false).unwrap();

        let params = JsonRpcParams::default()
            .method("eth_getBlockByHash")
            .params(vec![hash, bool]);

        let result = self
            .client
            .set_params(params)
            .send_json_rpc::<Block>()
            .await?;
        Ok(result)
    }

    pub async fn get_block(&self, hash: i64) -> crate::Result<Block> {
        let hash = serde_json::to_value(hash).unwrap();
        let bool = serde_json::to_value(true).unwrap();

        let params = JsonRpcParams::default()
            .method("eth_getBlockByNumber")
            .params(vec![hash, bool]);

        let result = self
            .client
            .set_params(params)
            .send_json_rpc::<Block>()
            .await?;
        Ok(result)
    }
}
