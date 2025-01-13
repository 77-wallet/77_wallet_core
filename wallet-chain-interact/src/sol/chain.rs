use super::{
    operations::{self},
    protocol::transaction::{CommitmentConfig, Status},
    provider::Provider,
    SolFeeSetting,
};
use crate::{
    sol::consts,
    types::{ChainPrivateKey, MultisigSignResp},
    BillResourceConsume, QueryTransactionResult,
};
use alloy::primitives::U256;
use solana_sdk::{
    compute_budget, instruction::Instruction, message::Message, signer::Signer,
    transaction::Transaction,
};

pub struct SolanaChain {
    provider: Provider,
}

impl SolanaChain {
    pub fn new(provider: Provider) -> crate::Result<Self> {
        Ok(Self { provider })
    }

    pub fn get_provider(&self) -> &Provider {
        &self.provider
    }
}

impl SolanaChain {
    pub async fn balance(&self, addr: &str, token: Option<String>) -> crate::Result<U256> {
        if let Some(t) = token
            && !t.is_empty()
        {
            let token_balance = self.provider.token_balance(&t, addr).await?;
            let balance = wallet_utils::unit::convert_to_u256(&token_balance.balance(), 0)?;
            Ok(balance)
        } else {
            let balance = self.provider.balance(addr).await?;
            Ok(U256::from(balance.value))
        }
    }

    pub async fn block_num(&self) -> crate::Result<u64> {
        self.provider.get_block_height().await
    }

    pub async fn decimals(&self, token_addr: &str) -> crate::Result<u8> {
        let supply = self.provider.total_supply(token_addr).await?;
        Ok(supply.value.decimals)
    }

    pub async fn token_symbol(&self, token: &str) -> crate::Result<String> {
        let symbol = self.provider.token_symbol(token).await?;
        Ok(symbol.chars().filter(|c| c.is_alphanumeric()).collect())
    }

    pub async fn token_name(&self, token: &str) -> crate::Result<String> {
        let name = self.provider.token_name(token).await?;
        Ok(name.chars().filter(|c| c.is_alphanumeric()).collect())
    }

    pub async fn black_address(&self, token: &str, owner: &str) -> crate::Result<bool> {
        let res = self.provider.token_balance(token, owner).await?;
        if res.value.is_empty() {
            tracing::warn!(
                "sol black address not found token account,token {},owner {}",
                token,
                owner
            );
            return Ok(false);
        }

        Ok(res.value[0].account.data.parsed.info.state == "frozen")
    }

    pub async fn query_tx_res(&self, hash: &str) -> crate::Result<Option<QueryTransactionResult>> {
        let transaction = self.provider.query_transaction(hash, "finalized").await;
        let transaction = match transaction {
            Ok(transaction) => transaction,
            Err(_err) => return Ok(None),
        };
        let state: i8 = match transaction.meta.status {
            Status::Ok(_) => 2,  // 成功
            Status::Err(_) => 3, // 成功
        };

        let resource_consume =
            BillResourceConsume::one_resource(transaction.meta.compute_units_consumed as u64)
                .to_json_str()?;
        let transaction_fee = transaction.meta.fee as f64 / super::consts::SOL_VALUE as f64;

        let res = QueryTransactionResult::new(
            hash.to_string(),
            transaction_fee,
            resource_consume,
            transaction.block_time,
            state,
            transaction.slot,
        );

        Ok(Some(res))
    }

    /// build empty instruction to get per signature fee
    pub async fn per_signature_fee(&self) -> crate::Result<SolFeeSetting> {
        let keypair = solana_sdk::signature::Keypair::from_base58_string(consts::TEMP_SOL_KEYPAIR);
        let payer = keypair.pubkey();

        let block_hash = self
            .provider
            .latest_blockhash(CommitmentConfig::Finalized)
            .await?;
        let message = Message::new_with_blockhash(&[], Some(&payer), &block_hash);

        let raw_message = wallet_utils::hex_func::bs64_encode(&message)?;
        let res = self.provider.message_fee(raw_message.as_str()).await?;

        Ok(SolFeeSetting::new(res.value, 0))
    }

    pub async fn exec_transaction<T>(
        &self,
        params: T,
        key: ChainPrivateKey,
        fee_setting: Option<SolFeeSetting>,
        mut instructions: Vec<Instruction>,
        retry: usize,
    ) -> crate::Result<String>
    where
        T: operations::SolTransferOperation,
    {
        let s = solana_sdk::signature::Keypair::from_base58_string(&key);
        let payer = params.payer()?;

        let other = params.other_keypair();
        let mut keypair = vec![];
        if !other.is_empty() {
            keypair.extend(&other);
        }
        keypair.push(&s);

        // add fee instruction
        if let Some(fee) = fee_setting {
            if let Some(priority) = fee.priority_fee_per_compute_unit {
                let unit_price =
                    compute_budget::ComputeBudgetInstruction::set_compute_unit_price(priority);
                let unit_limit = compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(
                    fee.compute_units_consumed as u32,
                );
                instructions.splice(0..0, vec![unit_limit, unit_price]);
            }
        }

        let res = if retry > 0 {
            self.provider
                .send_and_confirm_transaction(instructions, &payer, &keypair, retry)
                .await?
        } else {
            self.provider
                .execute_transaction(instructions, &payer, &keypair)
                .await?
        };

        Ok(res)
    }

    pub async fn estimate_fee_v1<T>(
        &self,
        instructions: &[Instruction],
        params: &T,
    ) -> crate::Result<SolFeeSetting>
    where
        T: operations::SolTransferOperation,
    {
        let payer = params.payer()?;

        let block_hash = self
            .provider
            .latest_blockhash(CommitmentConfig::Finalized)
            .await?;
        let message = Message::new_with_blockhash(instructions, Some(&payer), &block_hash);

        let raw_message = wallet_utils::hex_func::bs64_encode(&message)?;
        let res = self.provider.message_fee(raw_message.as_str()).await?;

        let mut sol_fee = SolFeeSetting::new(res.value, 100_000);
        if let Some(extra_fee) = params.extra_fee().await? {
            sol_fee.extra_fee = Some(extra_fee);
        }

        Ok(sol_fee)
    }

    pub async fn sign_with_res<T>(
        &self,
        instructions: Vec<Instruction>,
        params: T,
        key: ChainPrivateKey,
    ) -> crate::Result<MultisigSignResp>
    where
        T: operations::SolInstructionOperation + operations::SolTransferOperation,
    {
        let keypair = solana_sdk::signature::Keypair::from_base58_string(&key);
        let payer = params.payer()?;

        let block_hash = self
            .provider
            .latest_blockhash(CommitmentConfig::Finalized)
            .await?;

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer),
            &[&keypair],
            block_hash,
        );
        let raw_tx =
            solana_sdk::bs58::encode(wallet_utils::hex_func::bin_encode_bytes(&tx)?).into_string();
        let res = self.provider.send_transaction(&raw_tx, true).await?;

        Ok(MultisigSignResp::new_with_tx_hash(
            res,
            tx.signatures[0].to_string(),
        ))
    }
}
