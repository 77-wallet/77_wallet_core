use super::operations::multisig::{BtcMultisigRaw, MultisigAccountOpt, MultisigTransactionOpt};
use super::params::{FeeSetting, TransferResp};
use super::provider::{Provider, ProviderConfig};
use super::script::BtcScript;
use super::signature::{BtcSignature, SignatureCombiner};
use super::{network_convert, operations, protocol};
use crate::types::{ChainPrivateKey, FetchMultisigAddressResp, MultisigSignResp, MultisigTxResp};
use crate::{BillResourceConsume, QueryTransactionResult};
use alloy::primitives::map::HashMap;
use alloy::primitives::U256;
use bitcoin::key::{rand, Keypair, Secp256k1};
use bitcoin::taproot::TaprootBuilder;
use bitcoin::{consensus, Address, Amount, ScriptBuf, Transaction};
use wallet_types::chain::address::r#type::BtcAddressType;
use wallet_utils::{hex_func, unit};

pub struct BtcChain {
    provider: Provider,
    pub network: wallet_types::chain::network::NetworkKind,
}
impl BtcChain {
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

impl BtcChain {
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
        let status = if transaction.confirmations > 2 { 2 } else { 3 };

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

        let size = if params.spend_all {
            transaction_builder.spent_all_set_fee(fee_rate, params.to, params.address_type)?
        } else {
            // 找零和手续费配置
            transaction_builder.change_and_fee(
                fee_rate,
                params.change_address,
                params.address_type,
                params.value,
            )?
        };

        // 验证是否超过最大手续费配置
        let fee = fee_rate * size as u64;
        if transaction_builder.exceeds_max_fee(fee) {
            return Err(crate::UtxoError::ExceedsMaximum.into());
        }

        // 签名
        let utxo = transaction_builder.utxo.used_utxo_to_hash_map();
        let signer = BtcSignature::new(&key, utxo)?;
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

    // fee unit is btc
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

        let fee = bitcoin::Amount::from_float_in(fee, bitcoin::Denomination::Bitcoin)
            .map_err(|e| crate::Error::Other(e.to_string()))?;

        let mut transaction_builder = params.build_with_fee(utxo, fee)?;
        let utxo = transaction_builder.utxo.used_utxo_to_hash_map();

        let signer = BtcSignature::new(&key, utxo)?;
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

        let size = if params.spend_all {
            transaction_builder.spent_all_set_fee(
                fee_rate,
                params.to.clone(),
                params.address_type,
            )?
        } else {
            // 找零和手续费配置
            transaction_builder.change_and_fee(
                fee_rate,
                params.change_address,
                params.address_type,
                params.value,
            )?
        };

        Ok(FeeSetting { fee_rate, size })
    }

    pub async fn build_multisig_tx(
        &self,
        params: operations::transfer::TransferArg,
    ) -> crate::Result<MultisigTxResp> {
        let utxo = self
            .provider
            .utxos(&params.from.to_string(), self.network)
            .await?;

        let fee_rate = params.get_fee_rate(&self.provider, self.network).await?;

        let mut transaction_builder = params.build_transaction(utxo)?;

        let size = transaction_builder.change_and_fee(
            fee_rate,
            params.change_address,
            params.address_type,
            params.value,
        )?;

        let used_utxo = transaction_builder.utxo.used_utxo_to_hash_map();

        let fee = fee_rate * size as u64;
        if transaction_builder.exceeds_max_fee(fee) {
            return Err(crate::UtxoError::ExceedsMaximum.into());
        }

        let raw = BtcMultisigRaw {
            used_utxo,
            multisig_address: params.from.to_string(),
            raw_hex: consensus::encode::serialize_hex(&transaction_builder.transaction),
        };

        let raw_hex_str = raw.to_string()?;
        let resp = MultisigTxResp {
            tx_hash: "".to_string(),
            raw_data: raw_hex_str,
        };
        Ok(resp)
    }

    pub async fn sign_multisig_tx(
        &self,
        params: MultisigTransactionOpt,
        key: ChainPrivateKey,
    ) -> crate::Result<MultisigSignResp> {
        let raw_data = BtcMultisigRaw::from_hex_str(&params.raw_data)?;

        let bytes = hex_func::hex_decode(&raw_data.raw_hex)?;
        let transaction = consensus::deserialize::<Transaction>(&bytes)
            .map_err(|e| crate::Error::Other(e.to_string()))?;

        let script = ScriptBuf::from_hex(&params.script_hex)
            .map_err(|e| crate::Error::BtcScript(e.to_string()))?;

        let signer = BtcSignature::new(&key, raw_data.used_utxo)?;
        let sign = signer
            .multisig_sign_v1(params.address_type, script, transaction, &self.provider)
            .await?;

        let signature = hex_func::bincode_encode(&sign)?;
        let resp = MultisigSignResp::new(signature);

        Ok(resp)
    }

    pub async fn exec_multisig_tx(
        &self,
        params: MultisigTransactionOpt,
        signatures: Vec<String>,
        inner_key: String,
    ) -> crate::Result<TransferResp> {
        let raw_data = BtcMultisigRaw::from_hex_str(&params.raw_data)?;

        let bytes = hex_func::hex_decode(&raw_data.raw_hex)?;
        let mut transaction = consensus::deserialize::<Transaction>(&bytes)
            .map_err(|e| crate::Error::Other(e.to_string()))?;

        let redeem_script = ScriptBuf::from_hex(&params.script_hex)
            .map_err(|e| crate::Error::BtcScript(e.to_string()))?;

        let combiner = SignatureCombiner::new(signatures, redeem_script);

        match params.address_type {
            BtcAddressType::P2sh => combiner.p2sh(&mut transaction)?,
            BtcAddressType::P2shWsh => combiner.p2sh_wsh(&mut transaction)?,
            BtcAddressType::P2wsh => combiner.p2wsh(&mut transaction)?,
            BtcAddressType::P2trSh => combiner.p2tr_sh(&mut transaction, &inner_key)?,
            _ => {
                return Err(crate::Error::Other(format!(
                    "exec transaction not support multisig address type = {}",
                    params.address_type,
                )))
            }
        };

        // check balance
        let balance = self.balance(&params.from, None).await?;
        let value = unit::convert_to_u256(&params.value, super::consts::BTC_DECIMAL)?;
        if balance < value {
            return Err(crate::Error::UtxoError(
                crate::UtxoError::InsufficientBalance,
            ));
        }
        let remain_balance = Amount::from_sat((balance - value).to::<u64>());

        // check fee
        let fee_rate = self
            .provider
            .fetch_fee_rate(super::consts::FEE_RATE as u32, self.network)
            .await?;
        let size = transaction.vsize();
        let transaction_fee = fee_rate * size as u64;
        if remain_balance < transaction_fee {
            return Err(crate::Error::UtxoError(crate::UtxoError::InsufficientFee));
        }

        let hex_raw = consensus::encode::serialize_hex(&transaction);

        let tx_hash = self.provider.send_raw_transaction(&hex_raw).await?;
        Ok(TransferResp::new(tx_hash, fee_rate, size))
    }

    pub async fn multisig_address(
        &self,
        params: MultisigAccountOpt,
    ) -> crate::Result<FetchMultisigAddressResp> {
        let script = if params.address_type != BtcAddressType::P2trSh {
            BtcScript::multisig_script(params.threshold, &params.owners)?
        } else {
            BtcScript::multisig_p2tr_script(params.threshold, &params.owners)?
        };

        let network = network_convert(self.network);

        let (address, authority_address) = match params.address_type {
            BtcAddressType::P2sh => {
                let address = bitcoin::Address::p2sh(&script, network)
                    .map_err(|e| crate::Error::Other(e.to_string()))?;
                (address, "".to_string())
            }
            BtcAddressType::P2wsh => (Address::p2wsh(&script, network), "".to_string()),
            BtcAddressType::P2shWsh => (Address::p2shwsh(&script, network), "".to_string()),
            BtcAddressType::P2trSh => {
                let secp = Secp256k1::new();

                let keypair = Keypair::new(&secp, &mut rand::thread_rng());
                let (inner_pubkey, _) = keypair.x_only_public_key();

                let builder = TaprootBuilder::with_huffman_tree(vec![(1, script.clone())])
                    .map_err(|e| crate::Error::Other(e.to_string()))?;
                let tap_info = builder
                    .finalize(&secp, inner_pubkey)
                    .map_err(|e| crate::Error::Other(format!("{e:?}")))?;

                let address = Address::p2tr(
                    &secp,
                    tap_info.internal_key(),
                    tap_info.merkle_root(),
                    network,
                );
                (address, inner_pubkey.to_string())
            }
            _ => return Err(crate::Error::NotSupportApi("not support".to_string())),
        };

        let resp = FetchMultisigAddressResp {
            authority_address,
            multisig_address: address.to_string(),
            salt: script.to_hex_string(),
        };
        Ok(resp)
    }

    pub async fn decimals(&self, _token: &str) -> crate::Result<u8> {
        Ok(super::consts::BTC_DECIMAL)
    }

    pub async fn token_symbol(&self, _token: &str) -> crate::Result<String> {
        Ok("".to_string())
    }

    pub async fn token_name(&self, _token: &str) -> crate::Result<String> {
        Ok("".to_string())
    }
}
