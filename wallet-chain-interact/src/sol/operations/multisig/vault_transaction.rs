use borsh::{BorshDeserialize, BorshSerialize, to_vec};
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    instruction::{AccountMeta, Instruction},
    message::{AccountKeys, CompileError},
    pubkey::Pubkey,
};

use super::{compiled_keys::CompiledKeys, small_vec::SmallVec};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct TransactionMessage {
    /// The number of signer pubkeys in the account_keys vec.
    pub num_signers: u8,
    /// The number of writable signer pubkeys in the account_keys vec.
    pub num_writable_signers: u8,
    /// The number of writable non-signer pubkeys in the account_keys vec.
    pub num_writable_non_signers: u8,
    /// The list of unique account public keys (including program IDs) that will be used in the provided instructions.
    pub account_keys: SmallVec<u8, Pubkey>,
    /// The list of instructions to execute.
    pub instructions: SmallVec<u8, CompiledInstruction>,
    /// List of address table lookups used to load additional accounts
    /// for this transaction.
    pub address_table_lookups: SmallVec<u8, MessageAddressTableLookup>,
}

impl TransactionMessage {
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        to_vec(self).map_err(|e| crate::Error::Other(e.to_string()))
    }

    pub fn from_slice(data: &[u8]) -> crate::Result<Self> {
        let rs = borsh::from_slice::<Self>(data).unwrap();
        Ok(rs)
    }

    pub fn get_account_meta(&self) -> Vec<AccountMeta> {
        let mut account_meta = Vec::new();
        let account: Vec<Pubkey> = self.account_keys.clone().into();

        for i in &account {
            account_meta.push(AccountMeta::new(*i, false));
        }

        account_meta
    }
}

// Concise serialization schema for instructions that make up transaction.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CompiledInstruction {
    pub program_id_index: u8,
    /// Indices into the tx's `account_keys` list indicating which accounts to pass to the instruction.
    pub account_indexes: SmallVec<u8, u8>,
    /// Instruction data.
    pub data: SmallVec<u16, u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
/// Address table lookups describe an on-chain address lookup table to use
/// for loading more readonly and writable accounts in a single tx.
pub struct MessageAddressTableLookup {
    /// Address lookup table account key
    pub account_key: Pubkey,
    /// List of indexes used to load writable account addresses
    pub writable_indexes: SmallVec<u8, u8>,
    /// List of indexes used to load readonly account addresses
    pub readonly_indexes: SmallVec<u8, u8>,
}

pub trait VaultTransactionMessageExt {
    fn as_transaction_message(&self) -> &TransactionMessage;

    /// This implementation is mostly a copy-paste from `solana_program::message::v0::Message::try_compile()`,
    /// but it constructs a `TransactionMessage` meant to be passed to `vault_transaction_create`.
    fn try_compile(
        vault_key: &Pubkey,
        instructions: &[Instruction],
        address_lookup_table_accounts: &[AddressLookupTableAccount],
    ) -> Result<TransactionMessage, CompileError> {
        let mut compiled_keys = CompiledKeys::compile(instructions, Some(*vault_key));

        let mut address_table_lookups = Vec::with_capacity(address_lookup_table_accounts.len());
        let mut loaded_addresses_list = Vec::with_capacity(address_lookup_table_accounts.len());
        for lookup_table_account in address_lookup_table_accounts {
            if let Some((lookup, loaded_addresses)) =
                compiled_keys.try_extract_table_lookup(lookup_table_account)?
            {
                address_table_lookups.push(lookup);
                loaded_addresses_list.push(loaded_addresses);
            }
        }

        let (header, static_keys) = compiled_keys.try_into_message_components()?;
        let dynamic_keys = loaded_addresses_list.into_iter().collect();
        let account_keys = AccountKeys::new(&static_keys, Some(&dynamic_keys));
        let instructions = account_keys.try_compile_instructions(instructions)?;

        let num_static_keys: u8 = static_keys
            .len()
            .try_into()
            .map_err(|_| CompileError::AccountIndexOverflow)?;

        Ok(TransactionMessage {
            num_signers: header.num_required_signatures,
            num_writable_signers: header.num_required_signatures
                - header.num_readonly_signed_accounts,
            num_writable_non_signers: num_static_keys
                - header.num_required_signatures
                - header.num_readonly_unsigned_accounts,
            account_keys: static_keys.into(),
            instructions: instructions
                .into_iter()
                .map(|ix| CompiledInstruction {
                    program_id_index: ix.program_id_index,
                    account_indexes: ix.accounts.into(),
                    data: ix.data.into(),
                })
                .collect::<Vec<CompiledInstruction>>()
                .into(),
            address_table_lookups: address_table_lookups
                .into_iter()
                .map(|lookup| MessageAddressTableLookup {
                    account_key: lookup.account_key,
                    writable_indexes: lookup.writable_indexes.into(),
                    readonly_indexes: lookup.readonly_indexes.into(),
                })
                .collect::<Vec<MessageAddressTableLookup>>()
                .into(),
        })
    }

    // fn get_accounts_for_execute(
    //     &self,
    //     vault_pda: &Pubkey,
    //     transaction_pda: &Pubkey,
    //     address_lookup_table_accounts: &[AddressLookupTableAccount],
    //     num_ephemeral_signers: u8,
    //     program_id: &Pubkey,
    // ) -> Result<Vec<AccountMeta>, Error> {
    //     let message = VaultTransactionMessage::try_from(self.as_transaction_message().to_owned())
    //         .map_err(|_| Error::InvalidTransactionMessage)?;

    //     let ephemeral_signer_pdas: Vec<Pubkey> = (0..num_ephemeral_signers)
    //         .into_iter()
    //         .map(|ephemeral_signer_index| {
    //             get_ephemeral_signer_pda(transaction_pda, ephemeral_signer_index, Some(program_id))
    //                 .0
    //         })
    //         .collect();

    //     // region: -- address_lookup_tables map --

    //     let address_lookup_tables = address_lookup_table_accounts
    //         .into_iter()
    //         .map(|alt| (alt.key, alt))
    //         .collect::<HashMap<_, _>>();

    //     // endregion: -- address_lookup_tables map --

    //     // region: -- Account Metas --

    //     // First go the lookup table accounts used by the transaction. They are needed for on-chain validation.
    //     let lookup_table_account_metas = address_lookup_table_accounts
    //         .iter()
    //         .map(|alt| AccountMeta {
    //             pubkey: alt.key,
    //             is_writable: false,
    //             is_signer: false,
    //         })
    //         .collect::<Vec<_>>();

    //     // Then come static account keys included into the message.
    //     let static_account_metas = message
    //         .account_keys
    //         .iter()
    //         .enumerate()
    //         .map(|(index, &pubkey)| {
    //             AccountMeta {
    //                 pubkey,
    //                 is_writable: message.is_static_writable_index(index),
    //                 // NOTE: vaultPda and ephemeralSignerPdas cannot be marked as signers,
    //                 // because they are PDAs and hence won't have their signatures on the transaction.
    //                 is_signer: message.is_signer_index(index)
    //                     && &pubkey != vault_pda
    //                     && !ephemeral_signer_pdas.contains(&pubkey),
    //             }
    //         })
    //         .collect::<Vec<_>>();

    //     // And the last go the accounts that will be loaded with address lookup tables.
    //     let loaded_account_metas = message
    //         .address_table_lookups
    //         .iter()
    //         .map(|lookup| {
    //             let lookup_table_account = address_lookup_tables
    //                 .get(&lookup.account_key)
    //                 .ok_or(Error::InvalidAddressLookupTableAccount)?;

    //             // For each lookup, fist list writable, then readonly account metas.
    //             lookup
    //                 .writable_indexes
    //                 .iter()
    //                 .map(|&index| {
    //                     let pubkey = lookup_table_account
    //                         .addresses
    //                         .get(index as usize)
    //                         .ok_or(Error::InvalidAddressLookupTableAccount)?
    //                         .to_owned();

    //                     Ok(AccountMeta {
    //                         pubkey,
    //                         is_writable: true,
    //                         is_signer: false,
    //                     })
    //                 })
    //                 .chain(lookup.readonly_indexes.iter().map(|&index| {
    //                     let pubkey = lookup_table_account
    //                         .addresses
    //                         .get(index as usize)
    //                         .ok_or(Error::InvalidAddressLookupTableAccount)?
    //                         .to_owned();

    //                     Ok(AccountMeta {
    //                         pubkey,
    //                         is_writable: false,
    //                         is_signer: false,
    //                     })
    //                 }))
    //                 .collect::<Result<Vec<_>, Error>>()
    //         })
    //         .collect::<Result<Vec<_>, Error>>()?
    //         .into_iter()
    //         .flatten()
    //         .collect::<Vec<_>>();

    //     // endregion: -- Account Metas --

    //     Ok([
    //         lookup_table_account_metas,
    //         static_account_metas,
    //         loaded_account_metas,
    //     ]
    //     .concat())
    // }
}

impl VaultTransactionMessageExt for TransactionMessage {
    fn as_transaction_message(&self) -> &TransactionMessage {
        self
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VaultTransactionMessage {
    /// The number of signer pubkeys in the account_keys vec.
    pub num_signers: u8,
    /// The number of writable signer pubkeys in the account_keys vec.
    pub num_writable_signers: u8,
    /// The number of writable non-signer pubkeys in the account_keys vec.
    pub num_writable_non_signers: u8,
    /// Unique account pubkeys (including program IDs) required for execution of the tx.
    /// The signer pubkeys appear at the beginning of the vec, with writable pubkeys first, and read-only pubkeys following.
    /// The non-signer pubkeys follow with writable pubkeys first and read-only ones following.
    /// Program IDs are also stored at the end of the vec along with other non-signer non-writable pubkeys:
    ///
    /// ```plaintext
    /// [pubkey1, pubkey2, pubkey3, pubkey4, pubkey5, pubkey6, pubkey7, pubkey8]
    ///  |---writable---|  |---readonly---|  |---writable---|  |---readonly---|
    ///  |------------signers-------------|  |----------non-singers-----------|
    /// ```
    pub account_keys: Vec<Pubkey>,
    /// List of instructions making up the tx.
    pub instructions: Vec<MultisigCompiledInstruction>,
    /// List of address table lookups used to load additional accounts
    /// for this transaction.
    pub address_table_lookups: Vec<MultisigMessageAddressTableLookup>,
}

impl TryFrom<TransactionMessage> for VaultTransactionMessage {
    type Error = crate::Error;

    fn try_from(message: TransactionMessage) -> crate::Result<Self> {
        let account_keys: Vec<Pubkey> = message.account_keys.into();
        let instructions: Vec<CompiledInstruction> = message.instructions.into();
        let instructions: Vec<MultisigCompiledInstruction> = instructions
            .into_iter()
            .map(MultisigCompiledInstruction::from)
            .collect();
        let address_table_lookups: Vec<MessageAddressTableLookup> =
            message.address_table_lookups.into();

        Ok(Self {
            num_signers: message.num_signers,
            num_writable_signers: message.num_writable_signers,
            num_writable_non_signers: message.num_writable_non_signers,
            account_keys,
            instructions,
            address_table_lookups: address_table_lookups
                .into_iter()
                .map(MultisigMessageAddressTableLookup::from)
                .collect(),
        })
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MultisigCompiledInstruction {
    pub program_id_index: u8,
    /// Indices into the tx's `account_keys` list indicating which accounts to pass to the instruction.
    pub account_indexes: Vec<u8>,
    /// Instruction data.
    pub data: Vec<u8>,
}

impl From<CompiledInstruction> for MultisigCompiledInstruction {
    fn from(compiled_instruction: CompiledInstruction) -> Self {
        Self {
            program_id_index: compiled_instruction.program_id_index,
            account_indexes: compiled_instruction.account_indexes.into(),
            data: compiled_instruction.data.into(),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MultisigMessageAddressTableLookup {
    /// Address lookup table account key.
    pub account_key: Pubkey,
    /// List of indexes used to load writable accounts.
    pub writable_indexes: Vec<u8>,
    /// List of indexes used to load readonly accounts.
    pub readonly_indexes: Vec<u8>,
}

impl From<MessageAddressTableLookup> for MultisigMessageAddressTableLookup {
    fn from(m: MessageAddressTableLookup) -> Self {
        Self {
            account_key: m.account_key,
            writable_indexes: m.writable_indexes.into(),
            readonly_indexes: m.readonly_indexes.into(),
        }
    }
}
