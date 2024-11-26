use crate::sol::{operations::contract::TokenTransferBuild, provider::Provider};
use async_trait::async_trait;
use solana_sdk::program_pack::Pack;
use spl_associated_token_account::get_associated_token_address;
use wallet_utils::address;

pub struct TransferOpt<'a> {
    pub from: solana_sdk::pubkey::Pubkey,
    pub to: solana_sdk::pubkey::Pubkey,
    pub value: u64,
    pub token: Option<solana_sdk::pubkey::Pubkey>,
    pub decimal: u8,
    pub provider: &'a Provider,
}

impl<'a> TransferOpt<'a> {
    pub fn new(
        from: &str,
        to: &str,
        value: &str,
        token: Option<String>,
        decimal: u8,
        provider: &'a Provider,
    ) -> crate::Result<Self> {
        let value = wallet_utils::unit::convert_to_u256(value, decimal)?;

        let token = match token {
            Some(token) => Some(address::parse_sol_address(&token)?),
            None => None,
        };

        Ok(Self {
            from: address::parse_sol_address(from)?,
            to: address::parse_sol_address(to)?,
            value: value.to::<u64>(),
            token,
            decimal,
            provider,
        })
    }
}

#[async_trait]
impl super::SolInstructionOperation for TransferOpt<'_> {
    async fn instructions(&self) -> crate::Result<Vec<solana_sdk::instruction::Instruction>> {
        let instructions = if let Some(token) = self.token {
            let token_build = TokenTransferBuild::new(self, token).unwrap();
            token_build.transfer_instruction().await?
        } else {
            vec![solana_sdk::system_instruction::transfer(
                &self.from, &self.to, self.value,
            )]
        };
        Ok(instructions)
    }
}

#[async_trait]
impl super::SolTransferOperation for TransferOpt<'_> {
    fn payer(&self) -> crate::Result<solana_sdk::pubkey::Pubkey> {
        Ok(self.from)
    }

    /// In an SPL token transfer, if the recipient’s token address does not exist, additional fees will be incurred to create the account.
    async fn extra_fee(&self) -> crate::Result<Option<u64>> {
        if let Some(token) = self.token {
            let destination_pubkey = get_associated_token_address(&self.to, &token);

            // Check whether the address has a token account.
            let to_account = self.provider.account_info(destination_pubkey).await?;

            if to_account.value.is_none() {
                let data_len = spl_token_2022::state::Account::LEN;
                let value = self
                    .provider
                    .get_minimum_balance_for_rent(data_len as u64)
                    .await?;
                return Ok(Some(value));
            }
        }
        Ok(None)
    }
}
