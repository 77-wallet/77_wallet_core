use super::{
    operations::multisig::program::{MultisigArgs, ProgramConfig},
    protocol::{
        account::{AccountInfo, Balance, TokenAccount},
        contract::TotalSupply,
        transaction::{CommitmentConfig, Status},
    },
};
use crate::sol::protocol::{
    block::{Block, BlockHash},
    transaction::{SignatureStatus, TransactionResponse},
    Response,
};
use serde_json::json;
use solana_sdk::{
    hash::Hash, instruction::Instruction, pubkey::Pubkey, signature::Keypair,
    transaction::Transaction,
};
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;
use wallet_transport::{
    client::RpcClient,
    types::{JsonRpcParams, JsonRpcResult},
};

pub struct Provider {
    pub client: RpcClient,
}

impl Provider {
    pub fn new(rpc_client: RpcClient) -> crate::Result<Self> {
        Ok(Self { client: rpc_client })
    }

    pub async fn balance(&self, address: &str) -> crate::Result<Balance> {
        let params = JsonRpcParams::default()
            .method("getBalance")
            .params(vec![address]);

        let res = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<Balance>>()
            .await?;
        Ok(res.result)
    }

    pub async fn token_balance(&self, token: &str, address: &str) -> crate::Result<TokenAccount> {
        let req = vec![
            address.into(),
            json!({
                "mint": token,
            }),
            json!({
                "encoding": "jsonParsed"
            }),
        ];

        let params = JsonRpcParams::default()
            .method("getTokenAccountsByOwner")
            .params(req);

        let res = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<TokenAccount>>()
            .await?;
        Ok(res.result)
    }

    pub async fn token_symbol(&self, mint: &str) -> crate::Result<String> {
        let program_id =
        // spl_associated_token_account::ID;
            wallet_utils::address::parse_sol_address(super::operations::contract::META_PRAMS_ID)?;
        let mint_pubkey = Pubkey::from_str(mint).map_err(|e| crate::Error::Other(e.to_string()))?;

        // Derive the metadata PDA (Program Derived Address)
        let metadata_pda = Pubkey::find_program_address(
            &[b"metadata", program_id.as_ref(), mint_pubkey.as_ref()],
            &program_id,
        )
        .0;
        // Fetch account info of the metadata account
        let account_info =
            self.account_info(metadata_pda)
                .await?
                .value
                .ok_or(crate::Error::Other(
                    "Metadata account not found".to_string(),
                ))?;

        // Ensure the account has data
        if account_info.data.is_empty() {
            return Err(crate::Error::Other("Empty metadata account".to_string()));
        }

        let data = wallet_utils::base64_to_bytes(&account_info.data[0])?;

        let metadata = mpl_token_metadata::accounts::Metadata::from_bytes(&data).unwrap();

        // Return the symbol from the metadata
        Ok(metadata.symbol)
    }

    pub async fn token_name(&self, mint: &str) -> crate::Result<String> {
        let program_id =
            wallet_utils::address::parse_sol_address(super::operations::contract::META_PRAMS_ID)?;
        let mint_pubkey = Pubkey::from_str(mint).map_err(|e| crate::Error::Other(e.to_string()))?;

        // Derive the metadata PDA (Program Derived Address)
        let metadata_pda = Pubkey::find_program_address(
            &[b"metadata", program_id.as_ref(), mint_pubkey.as_ref()],
            &program_id,
        )
        .0;

        // Fetch account info of the metadata account
        let account_info =
            self.account_info(metadata_pda)
                .await?
                .value
                .ok_or(crate::Error::Other(
                    "Metadata account not found".to_string(),
                ))?;

        // Ensure the account has data
        if account_info.data.is_empty() {
            return Err(crate::Error::Other("Empty metadata account".to_string()));
        }

        // let decoded_data = wallet_utils::address::bs58_addr_to_hex_bytes(account_info.data)?;
        let data = wallet_utils::base64_to_bytes(&account_info.data[0])?;

        let metadata = mpl_token_metadata::accounts::Metadata::from_bytes(&data).unwrap();

        Ok(metadata.name)
    }

    pub async fn get_transaction_index(&self, multisig_pda: &Pubkey) -> crate::Result<u64> {
        let account = self
            .account_info(*multisig_pda)
            .await?
            .value
            .ok_or(crate::Error::Other(
                "not found multisig account".to_string(),
            ))?;

        let multisig = account.data.first().unwrap();
        let multisig_pda = MultisigArgs::from_str(multisig)?;

        Ok(multisig_pda.stale_transaction_index)
    }

    pub async fn get_config_program(&self, config_pda: &Pubkey) -> crate::Result<ProgramConfig> {
        let account = self
            .account_info(*config_pda)
            .await?
            .value
            .ok_or(crate::Error::Other("not found config account".to_string()))?;

        let config = account.data.first().unwrap();
        let program_config = ProgramConfig::from_str(config)?;

        Ok(program_config)
    }

    pub async fn latest_block(
        &self,
        commitment: CommitmentConfig,
    ) -> crate::Result<Response<BlockHash>> {
        let params = JsonRpcParams::default()
            .method("getLatestBlockhash")
            .params(vec![json!({
                "commitment": commitment.to_string()
            })]);

        let res = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<Response<BlockHash>>>()
            .await
            .unwrap();

        Ok(res.result)
    }

    pub async fn latest_blockhash(&self, commitment: CommitmentConfig) -> crate::Result<Hash> {
        let block = self.latest_block(commitment).await?;

        let hash = Hash::from_str(&block.value.blockhash)
            .map_err(|e| crate::Error::Other(e.to_string()))?;

        Ok(hash)
    }

    // execute transaction
    pub async fn execute_transaction(
        &self,
        instructions: Vec<Instruction>,
        payer: &Pubkey,
        keypair: &[&Keypair],
    ) -> crate::Result<String> {
        let block_hash = self.latest_blockhash(CommitmentConfig::Finalized).await?;

        let tx =
            Transaction::new_signed_with_payer(&instructions, Some(payer), keypair, block_hash);

        let raw_tx =
            solana_sdk::bs58::encode(wallet_utils::hex_func::bin_encode_bytes(&tx)?).into_string();

        self.send_transaction(&raw_tx, true).await
    }

    // 发送 and 等待确认交易
    pub async fn send_and_confirm_transaction(
        &self,
        instructions: Vec<Instruction>,
        payer: &Pubkey,
        keypair: &[&Keypair],
        retries: usize,
    ) -> crate::Result<String> {
        let get_status_time = 600;

        for _ in 0..retries {
            // 执行交易
            let block_hash = self.latest_blockhash(CommitmentConfig::Processed).await?;

            let block_hash_str = block_hash.to_string();
            let tx =
                Transaction::new_signed_with_payer(&instructions, Some(payer), keypair, block_hash);
            let raw_tx = solana_sdk::bs58::encode(wallet_utils::hex_func::bin_encode_bytes(&tx)?)
                .into_string();

            let tx_hash = self.send_transaction(&raw_tx, false).await?;

            // query reuslt
            for _ in 0..get_status_time {
                sleep(Duration::from_millis(500)).await;

                match self.get_signature_status(&tx_hash).await? {
                    Some(res) => match res.status {
                        Status::Ok(_) => {
                            // 验证确认数量
                            if res.confirmation_status == CommitmentConfig::Confirmed.to_string() {
                                return Ok(tx_hash);
                            }
                        }
                        Status::Err(e) => {
                            let error_msg = wallet_utils::serde_func::serde_to_string(&e)?;
                            return Err(crate::Error::TransferError(error_msg));
                        }
                    },
                    None => {
                        // 验证blockhash有效
                        if !self
                            .is_blockhash_vaild(&block_hash_str, CommitmentConfig::Processed)
                            .await?
                        {
                            if self
                                .query_transaction(
                                    &tx_hash,
                                    CommitmentConfig::Finalized.to_string(),
                                )
                                .await
                                .is_ok()
                            {
                                return Ok(tx_hash);
                            } else {
                                // 交易查询失败，跳出内层循环，准备重试发送交易
                                break;
                            }
                        }
                    }
                }
            }

            if self
                .query_transaction(&tx_hash, CommitmentConfig::Finalized.to_string())
                .await
                .is_ok()
            {
                return Ok(tx_hash);
            }
        }

        Err(crate::Error::TransferError(format!(
            "failed to tranfer and retry {}",
            retries
        )))
    }

    pub async fn get_signature_status(
        &self,
        tx_hash: &str,
    ) -> crate::Result<Option<SignatureStatus>> {
        let params = JsonRpcParams::default()
            .method("getSignatureStatuses")
            .params(vec![vec![tx_hash]]);

        let res = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<Response<Vec<Option<SignatureStatus>>>>>()
            .await?;

        Ok(res.result.value[0].clone())
    }

    pub async fn send_transaction(&self, tx: &str, node_retry: bool) -> crate::Result<String> {
        let req = if node_retry {
            json!([tx])
        } else {
            json!([
                tx,
                json!({
                     "maxRetries":0,
                     "preflightCommitment":CommitmentConfig::Processed.to_string(),
                })
            ])
        };

        let params = JsonRpcParams::default()
            .method("sendTransaction")
            .params(req);

        let res = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<String>>()
            .await?;

        Ok(res.result)
    }

    pub async fn is_blockhash_vaild(
        &self,
        blockhash: &str,
        commitment: CommitmentConfig,
    ) -> crate::Result<bool> {
        let req = json!([
            blockhash,
            json!({
                "commitment":commitment.to_string(),
            })
        ]);

        let params = JsonRpcParams::default()
            .method("isBlockhashValid")
            .params(req);

        let res = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<Response<bool>>>()
            .await?;

        Ok(res.result.value)
    }

    pub async fn _simulate_transaction(
        &self,
        instructions: Vec<Instruction>,
        payer: &Pubkey,
        keypair: &[&Keypair],
    ) -> crate::Result<String> {
        let block_hash = self.latest_blockhash(CommitmentConfig::Finalized).await?;

        let tx =
            Transaction::new_signed_with_payer(&instructions, Some(payer), keypair, block_hash);
        let raw_tx =
            solana_sdk::bs58::encode(wallet_utils::hex_func::bin_encode_bytes(&tx)?).into_string();

        let params = JsonRpcParams::default()
            .method("simulateTransaction")
            .params(vec![raw_tx]);

        let res = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<String>>()
            .await
            .unwrap();

        Ok(res.result)
    }

    pub async fn message_fee(&self, message: &str) -> crate::Result<Response<u64>> {
        let commitment = json!({
            "commitment": "finalized"
        });

        let params = JsonRpcParams::default()
            .method("getFeeForMessage")
            .params(vec![message.into(), commitment]);

        let res = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<Response<u64>>>()
            .await?;

        Ok(res.result)
    }

    pub async fn query_transaction(
        &self,
        txid: &str,
        commitment: &str,
    ) -> crate::Result<TransactionResponse> {
        let req = json!([
            txid,
            json!({
                "encoding": "json",
                "maxSupportedTransactionVersion":0,
                "rewards": false,
                commitment:commitment
            }),
        ]);
        let params = JsonRpcParams::default()
            .method("getTransaction")
            .params(req);

        let res = self
            .client
            .set_params(params)
            .send_json_rpc::<TransactionResponse>()
            .await?;
        Ok(res)
    }

    pub async fn get_block(&self, slot: u64) -> crate::Result<Block> {
        let req = json!([
            slot,
            json!({
                "encoding": "json",
                "maxSupportedTransactionVersion":0,
                "rewards": false,
            }),
        ]);
        let params = JsonRpcParams::default().method("getBlock").params(req);

        let res = self
            .client
            .set_params(params)
            .send_json_stream::<Block>()
            .await?;
        Ok(res)
    }

    pub async fn get_block_height(&self) -> crate::Result<u64> {
        let params: JsonRpcParams<()> = JsonRpcParams::default()
            .method("getBlockHeight")
            .no_params();
        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<u64>()
            .await?;
        Ok(r)
    }

    pub async fn get_slot(&self) -> crate::Result<u64> {
        let params: JsonRpcParams<()> = JsonRpcParams::default().method("getSlot").no_params();
        let r = self
            .client
            .set_params(params)
            .send_json_rpc::<u64>()
            .await?;
        Ok(r)
    }

    pub async fn total_supply(&self, token_addr: &str) -> crate::Result<Response<TotalSupply>> {
        let params = JsonRpcParams::default()
            .method("getTokenSupply")
            .params(vec![token_addr]);

        let res = self
            .client
            .set_params(params)
            .send_json_rpc::<Response<TotalSupply>>()
            .await?;
        Ok(res)
    }

    pub async fn account_info(&self, addr: Pubkey) -> crate::Result<Response<Option<AccountInfo>>> {
        let encoding = json!({
            "encoding": "base64",
        });
        let v = vec![addr.to_string().into(), encoding];
        let params = JsonRpcParams::default().method("getAccountInfo").params(v);

        let res = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<Response<Option<AccountInfo>>>>()
            .await?;

        Ok(res.result)
    }

    pub async fn get_minimum_balance_for_rent(&self, data_len: u64) -> crate::Result<u64> {
        let params = JsonRpcParams::default()
            .method("getMinimumBalanceForRentExemption")
            .params(vec![data_len]);

        let res = self
            .client
            .set_params(params)
            .send::<JsonRpcResult<u64>>()
            .await?;

        Ok(res.result)
    }
}
