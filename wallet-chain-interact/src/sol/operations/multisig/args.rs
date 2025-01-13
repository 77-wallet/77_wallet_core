use super::vault_transaction::{TransactionMessage, VaultTransactionMessage};
use borsh::BorshDeserialize;
use solana_sdk::borsh1::get_instance_packed_len;
use wallet_utils::address;

/// multisig program params
#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Debug)]
pub struct VaultTransactionCreateArgs {
    /// Index of the vault this transaction belongs to.
    pub vault_index: u8,
    /// Number of ephemeral signing PDAs required by the transaction.
    pub ephemeral_signers: u8,
    pub transaction_message: Vec<u8>,
    pub memo: Option<String>,
}

impl std::str::FromStr for VaultTransactionCreateArgs {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = wallet_utils::base64_to_bytes(s)?;
        let rs = borsh::from_slice::<Self>(&bytes).unwrap();
        Ok(rs)
    }
}
impl VaultTransactionCreateArgs {
    pub fn new(vault_index: u8, ephemeral_signers: u8, transaction_message: Vec<u8>) -> Self {
        Self {
            vault_index,
            ephemeral_signers,
            transaction_message,
            memo: None,
        }
    }
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        borsh::to_vec(self).map_err(|e| crate::Error::Other(e.to_string()))
    }

    pub fn size(transaction_message: &[u8], ephemeral_signers: u8) -> crate::Result<usize> {
        let args = VaultTransactionCreateArgs::deserialize(&mut &transaction_message[..])
            .map_err(|e| crate::ParseErr::SolMultisigArgs(e.to_string()))?;

        let message = TransactionMessage::deserialize(&mut args.transaction_message.as_ref())
            .map_err(|e| crate::ParseErr::SolMultisigArgs(e.to_string()))?;

        let messagge = VaultTransactionMessage::try_from(message)
            .map_err(|e| crate::ParseErr::SolMultisigArgs(e.to_string()))?;

        let message_len = get_instance_packed_len(&messagge).unwrap_or_default();
        Ok(
            8 +   // anchor account discriminator
            32 +  // multisig
            32 +  // creator
            8 +   // index
            1 +   // bump 
            1 +   // vault_index
            1 +   // vault_bump
            (4 + usize::from(ephemeral_signers)) +     // ephemeral_signers_bumps vec
            message_len, // message
        )
    }
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Debug)]
pub struct MultisigCreateArgsV2 {
    pub config_authority: Option<solana_sdk::pubkey::Pubkey>,
    pub threshold: u16,
    pub members: Vec<Member>,
    pub time_lock: u32,
    pub rent_collector: Option<solana_sdk::pubkey::Pubkey>,
    pub memo: Option<String>,
}
impl MultisigCreateArgsV2 {
    pub fn new(
        from: solana_program::pubkey::Pubkey,
        threshold: u16,
        members: Vec<String>,
    ) -> crate::Result<Self> {
        let mut member = vec![];
        for item in members.iter() {
            let i = Member {
                key: address::parse_sol_address(item)?,
                permissions: Permissions { mask: 7 },
            };
            member.push(i);
        }

        Ok(Self {
            config_authority: None,
            threshold,
            members: member,
            time_lock: 10,
            rent_collector: Some(from),
            memo: None,
        })
    }

    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        borsh::to_vec(self).map_err(|e| crate::Error::Other(e.to_string()))
    }
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Debug, serde::Deserialize)]
pub struct Member {
    pub key: solana_sdk::pubkey::Pubkey,
    pub permissions: Permissions,
}
#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Debug, serde::Deserialize)]
pub struct Permissions {
    pub mask: u8,
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Debug)]
pub struct ProposalCreateArgs {
    /// Index of the multisig transaction this proposal is associated with.
    pub transaction_index: u64,
    /// Whether the proposal should be initialized with status `Draft`.
    pub draft: bool,
}
impl ProposalCreateArgs {
    pub fn new(transaction_index: u64) -> Self {
        Self {
            transaction_index,
            draft: false,
        }
    }
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        borsh::to_vec(self).map_err(|e| crate::Error::Other(e.to_string()))
    }

    pub fn size(members_len: usize) -> usize {
        8 +   // anchor account discriminator
        32 +  // multisig
        8 +   // index
        1 +   // status enum variant
        8 +   // status enum wrapped timestamp (i64)
        1 +   // bump
        (4 + (members_len * 32)) + // approved vec
        (4 + (members_len * 32)) + // rejected vec
        (4 + (members_len * 32)) // cancelled vec
    }
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Debug)]
pub struct ProposalVoteArgs {
    pub memo: Option<String>,
}

impl Default for ProposalVoteArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl ProposalVoteArgs {
    pub fn new() -> Self {
        Self { memo: None }
    }
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        borsh::to_vec(self).map_err(|e| crate::Error::Other(e.to_string()))
    }
}
