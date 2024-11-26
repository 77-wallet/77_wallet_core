pub mod contract;
pub mod multisig;
pub mod transfer;

// to build instruction
#[async_trait::async_trait]
pub trait SolInstructionOperation {
    async fn instructions(&self) -> crate::Result<Vec<solana_sdk::instruction::Instruction>>;
    // fn other_keypair(&self) -> Vec<solana_sdk::signature::Keypair> {
    //     vec![]
    // }
    // fn payer(&self) -> crate::Result<solana_sdk::pubkey::Pubkey>;
    // async fn extra_fee(&self) -> crate::Result<Option<u64>>;
}

#[async_trait::async_trait]
pub trait SolTransferOperation {
    fn other_keypair(&self) -> Vec<solana_sdk::signature::Keypair> {
        vec![]
    }
    fn payer(&self) -> crate::Result<solana_sdk::pubkey::Pubkey>;

    async fn extra_fee(&self) -> crate::Result<Option<u64>>;
}
