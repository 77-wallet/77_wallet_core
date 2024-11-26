use super::{args::MultisigCreateArgsV2, pda, MULTISIG_PROGRAM_ID};
use crate::{
    sol::{
        operations::{SolInstructionOperation, SolTransferOperation},
        provider::Provider,
    },
    types::FetchMultisigAddressResp,
};
use async_trait::async_trait;
use solana_sdk::{instruction::AccountMeta, signer::Signer, system_program};
use wallet_utils::address;

// multisig account
pub struct MultisigAccountOpt<'a> {
    pub from: solana_sdk::pubkey::Pubkey,
    pub threshold: u8,
    pub owners: Vec<String>,
    pub salt: String,
    pub provider: &'a Provider,
}

impl<'a> MultisigAccountOpt<'a> {
    pub fn new(
        from: &str,
        threshold: u8,
        owners: Vec<String>,
        salt: String,
        provider: &'a Provider,
    ) -> crate::Result<Self> {
        let from = address::parse_sol_address(from)?;
        Ok(Self {
            from,
            threshold,
            owners,
            salt,
            provider,
        })
    }

    pub async fn create_account_instruction(
        &self,
        data: Vec<u8>,
        program_id: solana_sdk::pubkey::Pubkey,
        creator_key: solana_sdk::pubkey::Pubkey,
        multisig_pda: solana_sdk::pubkey::Pubkey,
    ) -> solana_sdk::instruction::Instruction {
        solana_sdk::instruction::Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(multisig_pda, false),
                AccountMeta::new_readonly(creator_key, true),
                AccountMeta::new(self.from, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        }
    }

    // new version : config_program to set fee
    pub async fn create_account_with_config(
        &self,
        data: Vec<u8>,
        program_id: solana_sdk::pubkey::Pubkey,
        creator_key: solana_sdk::pubkey::Pubkey,
        multisig_pda: solana_sdk::pubkey::Pubkey,
    ) -> crate::Result<solana_sdk::instruction::Instruction> {
        let (config_pda, _) = pda::get_program_config_pda(&program_id);
        let config_program = self.provider.get_config_program(&config_pda).await?;

        Ok(solana_sdk::instruction::Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new_readonly(config_pda, false),
                AccountMeta::new(config_program.treasury, false),
                AccountMeta::new(multisig_pda, false),
                AccountMeta::new_readonly(creator_key, true),
                AccountMeta::new(self.from, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        })
    }

    pub fn size(&self, members_length: usize) -> usize {
        8  + // anchor account discriminator
        32 + // create_key
        32 + // config_authority
        2  + // threshold
        4  + // time_lock
        8  + // transaction_index
        8  + // stale_transaction_index
        1  + // rent_collector Option discriminator
        32 + // rent_collector (always 32 bytes, even if None, just to keep the realloc logic simpler)
        1  + // bump
        4  + // members vector length
        members_length * 33 // members
    }
}

#[async_trait]
impl SolInstructionOperation for MultisigAccountOpt<'_> {
    async fn instructions(&self) -> crate::Result<Vec<solana_sdk::instruction::Instruction>> {
        let program_id = address::parse_sol_address(MULTISIG_PROGRAM_ID)?;

        let key = solana_sdk::signature::Keypair::from_base58_string(&self.salt);
        let creator_key = key.pubkey();

        let (multisig_pda, _) = pda::get_multisig_pda(&creator_key, &program_id);

        let args =
            MultisigCreateArgsV2::new(self.from, self.threshold as u16, self.owners.clone())?;
        let mut data = super::get_selector("multisig_create_v2");
        data.extend(args.to_bytes()?);

        let instruction = self
            .create_account_with_config(data, program_id, creator_key, multisig_pda)
            .await?;

        // let instruction = solana_sdk::instruction::Instruction {
        //     program_id,
        //     accounts: vec![
        //         AccountMeta::new(multisig_pda, false),
        //         AccountMeta::new_readonly(creator_key, true),
        //         AccountMeta::new(self.from, true),
        //         AccountMeta::new_readonly(system_program::id(), false),
        //     ],
        //     data,
        // };

        Ok(vec![instruction])
    }
}
#[async_trait]
impl SolTransferOperation for MultisigAccountOpt<'_> {
    fn payer(&self) -> crate::Result<solana_sdk::pubkey::Pubkey> {
        Ok(self.from)
    }

    fn other_keypair(&self) -> Vec<solana_sdk::signature::Keypair> {
        let key = solana_sdk::signature::Keypair::from_base58_string(&self.salt);
        vec![key]
    }
    // program fee and pda account fee
    async fn extra_fee(&self) -> crate::Result<Option<u64>> {
        let program_id = address::parse_sol_address(MULTISIG_PROGRAM_ID)?;
        let (config_pda, _) = pda::get_program_config_pda(&program_id);
        let config_program = self.provider.get_config_program(&config_pda).await?;

        // config_program.multisig_creation_fee +
        let account_fee = self
            .provider
            .get_minimum_balance_for_rent(self.size(self.owners.len()) as u64)
            .await?;

        Ok(Some(config_program.multisig_creation_fee + account_fee))
    }
}

/// fetch multisig address
impl MultisigAccountOpt<'_> {
    pub fn multisig_address() -> crate::Result<FetchMultisigAddressResp> {
        let create_keypair = solana_sdk::signature::Keypair::new();
        let create_key = create_keypair.pubkey();

        let program_id = address::parse_sol_address(MULTISIG_PROGRAM_ID)?;

        let (multisig_pda, _bump) = pda::get_multisig_pda(&create_key, &program_id);
        let (vault_pda, _) = pda::get_vault_pda(&multisig_pda, 0, &program_id);

        Ok(FetchMultisigAddressResp {
            multisig_address: vault_pda.to_string(),
            authority_address: multisig_pda.to_string(),
            salt: create_keypair.to_base58_string(),
        })
    }
}
