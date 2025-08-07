use super::operations;
use super::provider::Provider;
use crate::types::{ChainPrivateKey, FetchMultisigAddressResp, MultisigTxResp, Transaction};
use crate::{BillResourceConsume, QueryTransactionResult};
use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use alloy::sol_types::{SolType, SolValue};
use wallet_types::chain::chain::ChainCode;
use wallet_types::chain::network;
use wallet_utils::unit;

pub struct EthChain {
    pub provider: Provider,
    pub chain_code: ChainCode,
    network: network::NetworkKind,
}

impl EthChain {
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

impl EthChain {
    pub async fn balance(&self, addr: &str, token: Option<String>) -> crate::Result<U256> {
        if let Some(t) = token
            && !t.is_empty()
        {
            self.provider.token_balance(addr, &t).await
        } else {
            self.provider.balance(addr).await
        }
    }

    pub async fn block_num(&self) -> crate::Result<u64> {
        let block_height = self.provider.get_block_height().await?;
        let decimal_block_height =
            u64::from_str_radix(block_height.trim_start_matches("0x"), 16).unwrap();

        Ok(decimal_block_height)
    }

    pub async fn decimals(&self, token: &str) -> crate::Result<u8> {
        let res = self.provider.decimals(token).await?;
        Ok(res.to::<u8>())
    }

    pub async fn token_symbol(&self, token: &str) -> crate::Result<String> {
        let res = self.provider.token_symbol(token).await?;
        let decoded = wallet_utils::hex_func::hex_decode(&res[2..])? // 假设返回的是16进制编码的数据，需要解码为 UTF-8 字符串
            .into_iter()
            .filter(|&b| b.is_ascii_graphic()) // 过滤掉无效的非 ASCII 可打印字符
            .map(|b| b as char)
            .collect::<String>();
        Ok(decoded)
    }

    pub async fn token_name(&self, token: &str) -> crate::Result<String> {
        let res: String = self.provider.token_name(token).await?;
        let decoded = wallet_utils::hex_func::hex_decode(&res[2..])? // 假设返回的是16进制编码的数据，需要解码为 UTF-8 字符串
            .into_iter()
            .filter(|&b| b.is_ascii_graphic()) // 过滤掉无效的非 ASCII 可打印字符
            .map(|b| b as char)
            .collect::<String>();
        Ok(decoded)
    }

    pub async fn black_address(&self, token: &str, owner: &str) -> crate::Result<bool> {
        let res = self.provider.black_address(token, owner).await?;

        match res.as_str() {
            "0x0000000000000000000000000000000000000000000000000000000000000001" => Ok(true),
            "0x0000000000000000000000000000000000000000000000000000000000000000" => Ok(false),
            _ => Err(crate::Error::Other(
                "Invalid ABI encoding for boolean".to_string(),
            )),
        }
    }

    // 查询交易结果
    pub async fn query_tx_res(&self, hash: &str) -> crate::Result<Option<QueryTransactionResult>> {
        let receipt = match self.provider.transaction_receipt(hash).await {
            Ok(receipt) => receipt,
            Err(_err) => return Ok(None),
        };

        let hash = receipt.block_hash.unwrap_or_default().to_string();
        let block: alloy::rpc::types::Block = self.provider.block_by_hash(&hash).await?;

        let transaction_time = block.header.timestamp as u128;
        let transaction_fee =
            (receipt.effective_gas_price * receipt.gas_used) as f64 / super::consts::ETH_VALUE;
        let status = if receipt.status() { 2 } else { 3 };
        let block_number = block.header.number as u128;

        let resource_consume =
            BillResourceConsume::one_resource(receipt.gas_used as u64).to_json_str()?;
        let res = QueryTransactionResult::new(
            receipt.transaction_hash.to_string(),
            transaction_fee,
            resource_consume,
            transaction_time,
            status,
            block_number,
        );

        Ok(Some(res))
    }
}

// about estimate gas
impl EthChain {
    pub async fn estimate_gas<T>(&self, params: T) -> crate::Result<crate::params::ResourceConsume>
    where
        T: crate::types::Transaction<TransactionRequest>,
    {
        let params = params.build_transaction()?;

        let gas = self.provider.estimate_gas(params).await?;
        let res = crate::params::ResourceConsume::new(gas.to::<i64>());
        Ok(res)
    }
}

// about send transaction
impl EthChain {
    pub async fn eth_call<T, R>(&self, params: T) -> crate::Result<R>
    where
        T: crate::types::Transaction<TransactionRequest>,
        R: SolValue + std::convert::From<<<R as SolValue>::SolType as SolType>::RustType>,
    {
        let params = params.build_transaction()?;
        let res = self.provider.eth_call(params).await?;

        let input = res.strip_prefix("0x").unwrap_or(&res);
        let bytes = wallet_utils::hex_func::hex_decode(input)?;

        let result = R::abi_decode(&bytes, true)
            .map_err(|_e| crate::Error::HexError("abi code to address error".to_string()))?;

        Ok(result)
    }

    pub async fn exec_transaction<T>(
        &self,
        params: T,
        fee: super::params::FeeSetting,
        private_key: ChainPrivateKey,
    ) -> crate::Result<String>
    where
        T: crate::types::Transaction<TransactionRequest>,
    {
        let params = params.build_transaction()?;

        let fee = match self.network {
            network::NetworkKind::Mainnet => fee,
            _ => self.provider.get_fee(params.clone()).await?,
        };

        let transfer_params = self
            .provider
            .set_transaction_fee(params, fee, self.chain_code)
            .await?;

        self.provider
            .send_raw_transaction(transfer_params, &private_key)
            .await
    }
}

// about multisig
impl EthChain {
    pub async fn multisig_account(
        &self,
        params: operations::MultisigAccountOpt,
    ) -> crate::Result<FetchMultisigAddressResp> {
        let salt = params.get_nonce()?.to_string();
        let address: Address = self.eth_call(params).await?;
        Ok(FetchMultisigAddressResp::new_with_salt(
            address.to_string(),
            salt,
        ))
    }

    pub async fn build_multisig_tx(
        &self,
        mut params: operations::MultisigTransferOpt,
    ) -> crate::Result<MultisigTxResp> {
        let nonce_tx = params.nonce_tx()?;

        let nonce = self.provider.eth_call(nonce_tx).await?;
        params.nonce = unit::u256_from_str(&nonce)?;

        let transaction_request = params.build_transaction()?;
        let input_data = transaction_request.input.input().unwrap().to_string();

        let res = self.provider.eth_call(transaction_request).await?;
        let raw = operations::MultisigPayloadOpt::new(input_data, res.clone());

        Ok(MultisigTxResp {
            tx_hash: res,
            raw_data: raw.to_string()?,
        })
    }
}
