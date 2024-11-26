use super::{
    args::{ProposalCreateArgs, ProposalVoteArgs, VaultTransactionCreateArgs},
    pda,
    program::MultisigSigRawData,
    vault_transaction::{TransactionMessage, VaultTransactionMessageExt},
    MULTISIG_PROGRAM_ID,
};
use crate::{
    sol::{
        operations::{transfer::TransferOpt, SolInstructionOperation, SolTransferOperation},
        SolFeeSetting,
    },
    types::MultisigTxResp,
};
use solana_sdk::{instruction::AccountMeta, system_program};
use wallet_utils::address;

pub struct BuildTransactionArgs {
    pub transaction_index: u64,
    pub transaction_message: Vec<u8>,
}
impl BuildTransactionArgs {
    pub fn get_raw_data(
        &self,
        multisig_pda: solana_sdk::pubkey::Pubkey,
        tx_hash: String,
    ) -> crate::Result<MultisigTxResp> {
        let raw_str = wallet_utils::bytes_to_base64(&self.transaction_message);
        let raw_data = MultisigSigRawData::new(multisig_pda, raw_str, self.transaction_index);
        let tx = MultisigTxResp {
            tx_hash,
            raw_data: raw_data.to_base64_str()?,
        };
        Ok(tx)
    }
}

pub struct BuildTransactionOpt<'a> {
    pub multisig_pda: solana_sdk::pubkey::Pubkey,
    pub threshold: u8,
    pub creator: solana_sdk::pubkey::Pubkey,
    pub program_id: solana_sdk::pubkey::Pubkey,
    pub base: TransferOpt<'a>,
}
impl<'a> BuildTransactionOpt<'a> {
    pub fn new(
        multisig_pda: &str,
        threshold: u8,
        creator: &str,
        base: TransferOpt<'a>,
    ) -> crate::Result<Self> {
        Ok(Self {
            multisig_pda: address::parse_sol_address(multisig_pda)?,
            threshold,
            creator: address::parse_sol_address(creator)?,
            program_id: address::parse_sol_address(MULTISIG_PROGRAM_ID)?,
            base,
        })
    }

    // real transaction instructions
    pub async fn args_bytes(&self) -> crate::Result<Vec<u8>> {
        let transfer_instructions = self.base.instructions().await?;
        let vault_pda = self.base.from;

        let msg = TransactionMessage::try_compile(&vault_pda, &transfer_instructions, &[]).unwrap();
        let args = VaultTransactionCreateArgs::new(0, 0, msg.to_bytes()?);

        args.to_bytes()
    }

    // transaction bytes,transaction_index,
    pub async fn build_transaction_arg(&self) -> crate::Result<BuildTransactionArgs> {
        let transfer_instructions = self.base.instructions().await?;
        let vault_pda = self.base.from;

        let msg = TransactionMessage::try_compile(&vault_pda, &transfer_instructions, &[]).unwrap();
        let args = VaultTransactionCreateArgs::new(0, 0, msg.to_bytes()?);

        let transaction_index = self
            .base
            .provider
            .get_transaction_index(&self.multisig_pda)
            .await?;
        let transaction_index = transaction_index + 1;

        Ok(BuildTransactionArgs {
            transaction_index,
            transaction_message: args.to_bytes()?,
        })
    }

    pub async fn multisig_transfer_instruction(
        &self,
        args_bytes: &Vec<u8>,
        transaction_index: u64,
    ) -> crate::Result<solana_sdk::instruction::Instruction> {
        let mut data = super::get_selector("vault_transaction_create");
        data.extend(args_bytes);

        let (tx_pda, _) =
            pda::get_transaction_pda(&self.multisig_pda, transaction_index, &self.program_id);

        let instruction = solana_sdk::instruction::Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.multisig_pda, false),
                AccountMeta::new(tx_pda, false),
                AccountMeta::new_readonly(self.creator, true),
                AccountMeta::new(self.creator, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        };
        Ok(instruction)
    }

    // transaction proposal
    pub fn proposal_account_instruction(
        &self,
        transaction_index: u64,
    ) -> crate::Result<solana_sdk::instruction::Instruction> {
        let (proposal_pda, _) =
            pda::get_proposal_pda(&self.multisig_pda, transaction_index, &self.program_id);

        let args = ProposalCreateArgs::new(transaction_index);

        let mut data = super::get_selector("proposal_create");
        data.extend(args.to_bytes()?);

        let instruction = solana_sdk::instruction::Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.multisig_pda, false),
                AccountMeta::new(proposal_pda, false),
                AccountMeta::new_readonly(self.creator, true),
                AccountMeta::new(self.creator, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        };

        Ok(instruction)
    }

    pub async fn instructions_v0(
        &self,
        transaction_index: u64,
        args_bytes: &Vec<u8>,
    ) -> crate::Result<Vec<solana_sdk::instruction::Instruction>> {
        let transaction_instructions = self
            .multisig_transfer_instruction(args_bytes, transaction_index)
            .await?;

        let proposal_instruction = self.proposal_account_instruction(transaction_index)?;

        Ok(vec![transaction_instructions, proposal_instruction])
    }

    pub async fn instructions(
        &self,
        args: &BuildTransactionArgs,
    ) -> crate::Result<Vec<solana_sdk::instruction::Instruction>> {
        let transaction_instructions = self
            .multisig_transfer_instruction(&args.transaction_message, args.transaction_index)
            .await?;

        let proposal_instruction = self.proposal_account_instruction(args.transaction_index)?;

        Ok(vec![transaction_instructions, proposal_instruction])
    }

    // 额外的费用(lamports)
    pub async fn create_transaction_fee(
        &self,
        msg: &[u8],
        mut base_fee: SolFeeSetting,
    ) -> crate::Result<SolFeeSetting> {
        let vault_size = VaultTransactionCreateArgs::size(msg)?;

        // tracing::warn!("vault account size {}", vault_size);
        let proposal_size = ProposalCreateArgs::size(self.threshold as usize);

        let total_size = vault_size + proposal_size;

        let value = self
            .base
            .provider
            .get_minimum_balance_for_rent(total_size as u64)
            .await?;

        base_fee.extra_fee = Some(value);
        Ok(base_fee)
    }
}

#[async_trait::async_trait]
impl SolTransferOperation for BuildTransactionOpt<'_> {
    fn payer(&self) -> crate::Result<solana_sdk::pubkey::Pubkey> {
        Ok(self.creator)
    }
    async fn extra_fee(&self) -> crate::Result<Option<u64>> {
        Ok(None)
    }
}

// sign multisig transaction opt
pub struct SignTransactionOpt {
    pub singer: solana_sdk::pubkey::Pubkey,
    pub raw_data: String,
    pub program_id: solana_sdk::pubkey::Pubkey,
}
impl SignTransactionOpt {
    pub fn new(singer: &str, raw_data: String) -> crate::Result<Self> {
        Ok(Self {
            singer: address::parse_sol_address(singer)?,
            raw_data,
            program_id: address::parse_sol_address(MULTISIG_PROGRAM_ID)?,
        })
    }

    pub async fn instructions_v0(
        &self,
    ) -> crate::Result<Vec<solana_sdk::instruction::Instruction>> {
        let transaction = MultisigSigRawData::from_base64_str(&self.raw_data)?;
        let transaction_index = transaction.transaction_index;
        let multisig_pda = transaction.multisig_pda;

        let (proposal, _) =
            pda::get_proposal_pda(&multisig_pda, transaction_index, &self.program_id);
        let args = ProposalVoteArgs::new();

        let mut data = super::get_selector("proposal_approve");
        data.extend(args.to_bytes()?);

        let instruction = solana_sdk::instruction::Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new_readonly(multisig_pda, false),
                AccountMeta::new(self.singer, true),
                AccountMeta::new(proposal, false),
            ],
            data,
        };

        Ok(vec![instruction])
    }
}

// exec multisig transaction
#[async_trait::async_trait]
impl SolInstructionOperation for SignTransactionOpt {
    async fn instructions(&self) -> crate::Result<Vec<solana_sdk::instruction::Instruction>> {
        let transaction = MultisigSigRawData::from_base64_str(&self.raw_data)?;
        let transaction_index = transaction.transaction_index;
        let multisig_pda = transaction.multisig_pda;

        let (proposal, _) =
            pda::get_proposal_pda(&multisig_pda, transaction_index, &self.program_id);
        let args = ProposalVoteArgs::new();

        let mut data = super::get_selector("proposal_approve");
        data.extend(args.to_bytes()?);

        let instruction = solana_sdk::instruction::Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new_readonly(multisig_pda, false),
                AccountMeta::new(self.singer, true),
                AccountMeta::new(proposal, false),
            ],
            data,
        };

        Ok(vec![instruction])
    }
}

#[async_trait::async_trait]
impl SolTransferOperation for SignTransactionOpt {
    fn payer(&self) -> crate::Result<solana_sdk::pubkey::Pubkey> {
        Ok(self.singer)
    }

    async fn extra_fee(&self) -> crate::Result<Option<u64>> {
        Ok(None)
    }
}

// execute multisig op
pub struct ExecMultisigOpt {
    pub executor: solana_sdk::pubkey::Pubkey,
    pub raw_data: String,
    pub program_id: solana_sdk::pubkey::Pubkey,
}
impl ExecMultisigOpt {
    pub fn new(executor: &str, raw_data: String) -> crate::Result<Self> {
        Ok(Self {
            executor: address::parse_sol_address(executor)?,
            raw_data,
            program_id: address::parse_sol_address(MULTISIG_PROGRAM_ID)?,
        })
    }
}

// exec multisig transaction
#[async_trait::async_trait]
impl SolInstructionOperation for ExecMultisigOpt {
    async fn instructions(&self) -> crate::Result<Vec<solana_sdk::instruction::Instruction>> {
        let transaction = MultisigSigRawData::from_base64_str(&self.raw_data)?;

        let transaction_index = transaction.transaction_index;
        let multisig_pda = transaction.multisig_pda;

        let (proposal_pda, _) =
            pda::get_proposal_pda(&multisig_pda, transaction_index, &self.program_id);
        let (tx_pda, _) =
            pda::get_transaction_pda(&multisig_pda, transaction_index, &self.program_id);

        use std::str::FromStr as _;
        let args = VaultTransactionCreateArgs::from_str(&transaction.raw_data)?;
        let message = TransactionMessage::from_slice(&args.transaction_message)?;

        let account_meta = message.get_account_meta();
        let data = super::get_selector("vault_transaction_execute");

        let mut accounts = vec![
            AccountMeta::new_readonly(multisig_pda, false),
            AccountMeta::new(proposal_pda, false),
            AccountMeta::new_readonly(tx_pda, false),
            AccountMeta::new_readonly(self.executor, true),
        ];
        accounts.extend(account_meta);

        let instruction = solana_sdk::instruction::Instruction {
            program_id: self.program_id,
            accounts,
            data,
        };

        Ok(vec![instruction])
    }
}

#[async_trait::async_trait]
impl SolTransferOperation for ExecMultisigOpt {
    fn payer(&self) -> crate::Result<solana_sdk::pubkey::Pubkey> {
        Ok(self.executor)
    }

    async fn extra_fee(&self) -> crate::Result<Option<u64>> {
        Ok(None)
    }
}
