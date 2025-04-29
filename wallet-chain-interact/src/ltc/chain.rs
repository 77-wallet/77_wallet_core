use super::params::{FeeSetting, TransferResp};
use super::provider::{Provider, ProviderConfig};
use super::signature::LtcSignature;
use super::{operations, protocol};
use crate::types::ChainPrivateKey;
use crate::{BillResourceConsume, QueryTransactionResult};
use alloy::primitives::map::HashMap;
use alloy::primitives::U256;
use litecoin::Amount;

pub struct LtcChain {
    provider: Provider,
    pub network: wallet_types::chain::network::NetworkKind,
}
impl LtcChain {
    pub fn new(
        config: ProviderConfig,
        network: wallet_types::chain::network::NetworkKind,
        header_opt: Option<HashMap<String, String>>,
        timeout: Option<std::time::Duration>,
    ) -> crate::Result<Self> {
        let provider = Provider::new(config, header_opt, timeout)?;
        Ok(Self { provider, network })
    }

    pub fn get_provider(&self) -> &Provider {
        &self.provider
    }
}

impl LtcChain {
    pub async fn balance(&self, addr: &str, _token: Option<String>) -> crate::Result<U256> {
        let utxo = self.provider.utxos(addr, self.network).await?;
        Ok(U256::from(utxo.balance()))
    }

    pub async fn block_num(&self) -> crate::Result<u64> {
        let block_height = self.provider.block_heigh().await?;
        Ok(block_height)
    }

    // 查询交易结果
    pub async fn query_tx_res(&self, hash: &str) -> crate::Result<Option<QueryTransactionResult>> {
        let transaction = match self
            .provider
            .query_transaction::<protocol::transaction::Transaction>(hash, true)
            .await
        {
            Ok(res) => res,
            Err(_) => return Ok(None),
        };

        if transaction.blockhash.is_empty() {
            return Err(crate::Error::Other("transaction not confirm".to_string()));
        }

        // 获取区块的高度
        let block_header = self.provider.block_header(&transaction.blockhash).await?;

        // 查询上一个交易的总输出
        let mut total_vin = 0_f64;
        for vin in transaction.vin.iter() {
            let prev_tx = self
                .provider
                .query_transaction::<protocol::transaction::Transaction>(&vin.txid, true)
                .await?;
            total_vin += prev_tx.total_vout_by_sequence(vin.vout);
        }
        // 这次交易的总输出
        let total_vout = transaction.total_vout();

        let transaction_fee = total_vin - total_vout;
        let status = 2;

        // transaction.weight,
        let resource_consume =
            BillResourceConsume::one_resource(transaction.weight).to_json_str()?;
        let res = QueryTransactionResult::new(
            transaction.hash,
            transaction_fee,
            resource_consume,
            transaction.time as u128,
            status,
            block_header.height as u128,
        );
        Ok(Some(res))
    }

    pub async fn transfer(
        &self,
        params: operations::transfer::TransferArg,
        key: ChainPrivateKey,
    ) -> crate::Result<TransferResp> {
        let utxo = self
            .provider
            .utxos(&params.from.to_string(), self.network)
            .await?;
        let mut transaction_builder = params.build_transaction(utxo)?;

        let fee_rate = params.get_fee_rate(&self.provider, self.network).await?;

        let size = transaction_builder.transactin_size(fee_rate, &params)?;

        let fee = fee_rate * size as u64;
        if transaction_builder.exceeds_max_fee(fee) {
            return Err(crate::UtxoError::ExceedsMaximum.into());
        }

        if transaction_builder.is_dust_tx(params.value, fee) {
            return Err(crate::UtxoError::DustTx.into());
        }

        // 签名
        let utxo = transaction_builder.utxo.used_utxo_to_hash_map();
        let signer = LtcSignature::new(&key, utxo)?;
        signer
            .sign(
                params.address_type,
                &self.provider,
                &mut transaction_builder.transaction,
            )
            .await?;

        // 获取原始交易
        let raw = transaction_builder.get_raw_transaction();

        // 执行交易
        let tx_hash = self.provider.send_raw_transaction(&raw).await?;

        Ok(TransferResp::new(tx_hash, fee_rate, size))
    }

    // fee unit is ltc
    pub async fn transfer_with_fee(
        &self,
        params: operations::transfer::TransferArg,
        fee: f64,
        key: ChainPrivateKey,
    ) -> crate::Result<TransferResp> {
        let utxo = self
            .provider
            .utxos(&params.from.to_string(), self.network)
            .await?;

        let fee = litecoin::Amount::from_float_in(fee, litecoin::Denomination::Bitcoin)
            .map_err(|e| crate::Error::Other(e.to_string()))?;

        let mut transaction_builder = params.build_with_fee(utxo, fee)?;
        let utxo = transaction_builder.utxo.used_utxo_to_hash_map();

        let signer = LtcSignature::new(&key, utxo)?;
        signer
            .sign(
                params.address_type,
                &self.provider,
                &mut transaction_builder.transaction,
            )
            .await?;

        if transaction_builder.exceeds_max_fee(fee) {
            return Err(crate::UtxoError::ExceedsMaximum.into());
        }

        if transaction_builder.is_dust_tx(params.value, fee) {
            return Err(crate::UtxoError::DustTx.into());
        }
        let raw = transaction_builder.get_raw_transaction();

        // 执行交易
        let tx_hash = self.provider.send_raw_transaction(&raw).await?;

        Ok(TransferResp::new(tx_hash, Amount::default(), 0))
    }

    pub async fn estimate_fee(
        &self,
        params: operations::transfer::TransferArg,
    ) -> crate::Result<FeeSetting> {
        let utxo = self
            .provider
            .utxos(&params.from.to_string(), self.network)
            .await?;

        let mut transaction_builder = params.build_transaction(utxo)?;

        let fee_rate = params.get_fee_rate(&self.provider, self.network).await?;

        let size = transaction_builder.transactin_size(fee_rate, &params)?;

        Ok(FeeSetting { fee_rate, size })
    }

    pub async fn decimals(&self, _token: &str) -> crate::Result<u8> {
        Ok(super::consts::LTC_DECIMAL)
    }

    pub async fn token_symbol(&self, _token: &str) -> crate::Result<String> {
        Ok("".to_string())
    }

    pub async fn token_name(&self, _token: &str) -> crate::Result<String> {
        Ok("".to_string())
    }
}
