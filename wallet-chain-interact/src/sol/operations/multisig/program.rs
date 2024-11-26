use super::args::Member;

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Debug, serde::Deserialize)]
pub struct MultisigArgs {
    /// Key that is used to seed the multisig PDA.
    pub create_key: solana_sdk::pubkey::Pubkey,
    /// The authority that can change the multisig config.
    /// This is a very important parameter as this authority can change the members and threshold.
    ///
    /// The convention is to set this to `Pubkey::default()`.
    /// In this case, the multisig becomes autonomous, so every config change goes through
    /// the normal process of voting by the members.
    ///
    /// However, if this parameter is set to any other key, all the config changes for this multisig
    /// will need to be signed by the `config_authority`. We call such a multisig a "controlled multisig".
    pub config_authority: solana_sdk::pubkey::Pubkey,
    /// Threshold for signatures.
    pub threshold: u16,
    /// How many seconds must pass between transaction voting settlement and execution.
    pub time_lock: u32,
    /// Last transaction index. 0 means no transactions have been created.
    pub transaction_index: u64,
    /// Last stale transaction index. All transactions up until this index are stale.
    /// This index is updated when multisig config (members/threshold/time_lock) changes.
    pub stale_transaction_index: u64,
    /// The address where the rent for the accounts related to executed, rejected, or cancelled
    /// transactions can be reclaimed. If set to `None`, the rent reclamation feature is turned off.
    pub rent_collector: Option<solana_sdk::pubkey::Pubkey>,
    /// Bump for the multisig PDA seed.
    pub bump: u8,
    /// Members of the multisig.
    pub members: Vec<Member>,
}

impl std::str::FromStr for MultisigArgs {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = wallet_utils::base64_to_bytes(s)?;
        let data = solana_program::borsh1::try_from_slice_unchecked(&data).unwrap();
        Ok(data)
    }
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Debug)]
pub struct MultisigSigRawData {
    pub multisig_pda: solana_sdk::pubkey::Pubkey,
    pub transaction_index: u64,
    pub raw_data: String,
}

impl MultisigSigRawData {
    pub fn new(
        multisig_pda: solana_sdk::pubkey::Pubkey,
        raw_data: String,
        transaction_index: u64,
    ) -> Self {
        Self {
            multisig_pda,
            raw_data,
            transaction_index,
        }
    }

    pub fn to_base64_str(&self) -> crate::Result<String> {
        let bytes = borsh::to_vec(self).map_err(|e| crate::Error::Other(e.to_string()))?;
        Ok(wallet_utils::bytes_to_base64(&bytes))
    }

    pub fn from_base64_str(str: &str) -> crate::Result<Self> {
        let bytes = wallet_utils::base64_to_bytes(str)?;
        let rs = borsh::from_slice::<Self>(&bytes).unwrap();
        Ok(rs)
    }
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Debug)]
pub struct ProgramConfig {
    pub discriminator: [u8; 8],
    /// The authority which can update the config.
    pub authority: solana_program::pubkey::Pubkey,
    /// The lamports amount charged for creating a new multisig account.
    /// This fee is sent to the `treasury` account.
    pub multisig_creation_fee: u64,
    /// The treasury account to send charged fees to.
    pub treasury: solana_program::pubkey::Pubkey,
    /// Reserved for future use.
    pub _reserved: [u8; 64],
}

impl std::str::FromStr for ProgramConfig {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = wallet_utils::base64_to_bytes(s)?;
        let data = solana_program::borsh1::try_from_slice_unchecked(&data).unwrap();
        Ok(data)
    }
}
