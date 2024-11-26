use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use wallet_utils::address;

//  token param id
pub const TOKEN_PRAMS_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const META_PRAMS_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";

pub struct TokenTransferBuild<'a> {
    pub program_id: solana_sdk::pubkey::Pubkey,
    pub params: &'a super::transfer::TransferOpt<'a>,
    pub authority_pubkey: solana_sdk::pubkey::Pubkey,
    pub mint_pubkey: solana_sdk::pubkey::Pubkey,
}
impl<'a> TokenTransferBuild<'a> {
    pub fn new(
        params: &'a super::transfer::TransferOpt,
        mint_pubkey: solana_sdk::pubkey::Pubkey,
    ) -> crate::Result<Self> {
        let program_id = address::parse_sol_address(TOKEN_PRAMS_ID)?;

        Ok(Self {
            program_id,
            params,
            authority_pubkey: params.from,
            mint_pubkey,
        })
    }
}

impl<'a> TokenTransferBuild<'a> {
    // token transfer instruction
    pub async fn transfer_instruction(
        &self,
    ) -> crate::Result<Vec<solana_sdk::instruction::Instruction>> {
        let mut instruction = vec![];

        let source_pubkey = get_associated_token_address(&self.params.from, &self.mint_pubkey);
        let destination_pubkey = get_associated_token_address(&self.params.to, &self.mint_pubkey);

        // Check whether the address has a token account.
        let to_account = self
            .params
            .provider
            .account_info(destination_pubkey)
            .await?;
        if to_account.value.is_none() {
            instruction.push(self.associated_account_instruction());
        }

        let transfer = spl_token_2022::instruction::transfer_checked(
            &self.program_id,
            &source_pubkey,
            &self.mint_pubkey,
            &destination_pubkey,
            &self.authority_pubkey,
            &[],
            self.params.value,
            self.params.decimal,
        )
        .map_err(|e| crate::Error::Other(format!("build transfer instruction error:{}", e)))?;
        instruction.push(transfer);

        Ok(instruction)
    }

    // associated token account instruction
    pub fn associated_account_instruction(&self) -> solana_sdk::instruction::Instruction {
        create_associated_token_account(
            &self.authority_pubkey,
            &self.params.to,
            &self.mint_pubkey,
            &self.program_id,
        )
    }
}
