use crate::tron::Provider;

use super::{
    multisig::{MultisigAccountResp, Permission},
    RawTransactionParams, TronTransactionResponse, TronTxOperation,
};
use wallet_utils::address;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractType {
    AccountCreateContract = 0,
    TransferContract = 1,
    TransferAssetContract = 2,
    VoteAssetContract = 3,
    VoteWitnessContract = 4,
    WitnessCreateContract = 5,
    AssetIssueContract = 6,
    WitnessUpdateContract = 8,
    ParticipateAssetIssueContract = 9,
    AccountUpdateContract = 10,
    FreezeBalanceContract = 11,
    UnfreezeBalanceContract = 12,
    WithdrawBalanceContract = 13,
    UnfreezeAssetContract = 14,
    UpdateAssetContract = 15,
    ProposalCreateContract = 16,
    ProposalApproveContract = 17,
    ProposalDeleteContract = 18,
    SetAccountIdContract = 19,
    CustomContract = 20,
    CreateSmartContract = 30,
    TriggerSmartContract = 31,
    GetContract = 32,
    UpdateSettingContract = 33,
    ExchangeCreateContract = 41,
    ExchangeInjectContract = 42,
    ExchangeWithdrawContract = 43,
    ExchangeTransactionContract = 44,
    UpdateEnergyLimitContract = 45,
    AccountPermissionUpdateContract = 46,
    ClearABIContract = 48,
    UpdateBrokerageContract = 49,
    ShieldedTransferContract = 51,
    MarketSellAssetContract = 52,
    MarketCancelOrderContract = 53,
    FreezeBalanceV2Contract = 54,
    UnfreezeBalanceV2Contract = 55,
    WithdrawExpireUnfreezeContract = 56,
    DelegateResourceContract = 57,
    UnDelegateResourceContract = 58,
    CancelAllUnfreezeV2Contract = 59,
}

// 权限集合
pub struct PermissionTypes(pub Vec<ContractType>);

impl PermissionTypes {
    pub fn to_hex(&self) -> String {
        let contract_ids = self
            .0
            .iter()
            .map(|i| i.to_u8() as usize)
            .collect::<Vec<usize>>();

        let mut operations = [0u8; 32];
        for &id in &contract_ids {
            operations[id / 8] |= 1 << (id % 8);
        }

        hex::encode(operations)
    }
}

impl Default for PermissionTypes {
    fn default() -> Self {
        let contract_ids = vec![
            ContractType::AccountCreateContract,           // Activate Account
            ContractType::TransferContract,                // Transfer TRX
            ContractType::TransferAssetContract,           // Transfer TRC10
            ContractType::VoteWitnessContract,             // Vote
            ContractType::WitnessCreateContract,           // Apply to Become a SR Candidate
            ContractType::AssetIssueContract,              // Issue TRC10
            ContractType::WitnessUpdateContract,           // Update SR Info
            ContractType::ParticipateAssetIssueContract,   // Participate in TRC10 Issuance
            ContractType::AccountUpdateContract,           // Update Account Name
            ContractType::FreezeBalanceContract,           // TRX Stake (1.0)
            ContractType::UnfreezeBalanceContract,         // TRX Unstake (1.0)
            ContractType::WithdrawBalanceContract,         // Claim Voting Rewards
            ContractType::UnfreezeAssetContract,           // Unstake TRC10
            ContractType::UpdateAssetContract,             // Update TRC10 Parameters
            ContractType::ProposalCreateContract,          // Create Proposal
            ContractType::ProposalApproveContract,         // Approve Proposal
            ContractType::ProposalDeleteContract,          // Cancel Proposal
            ContractType::CreateSmartContract,             // Create Smart Contract
            ContractType::TriggerSmartContract,            // Trigger Smart Contract
            ContractType::UpdateSettingContract,           // Update Contract Parameters
            ContractType::ExchangeCreateContract,          // Create Bancor Transaction
            ContractType::ExchangeInjectContract,          // Inject Assets into Bancor Transaction
            ContractType::ExchangeWithdrawContract, // Withdraw Assets from Bancor Transaction
            ContractType::ExchangeTransactionContract, // Execute Bancor Transaction
            ContractType::UpdateEnergyLimitContract, // Update Contract Energy Limit
            ContractType::AccountPermissionUpdateContract, // Update Account Permissions
        ];
        println!("len {}", contract_ids.len());
        PermissionTypes(contract_ids)
    }
}

impl ContractType {
    pub fn to_u8(&self) -> i8 {
        *self as i8
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PermissionUpdateArgs {
    pub owner_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Permission>,
    // 权限
    pub actives: Option<Vec<Permission>>,
}

impl PermissionUpdateArgs {
    pub fn new(
        owner_address: &str,
        owner: Option<Permission>,
        actives: Option<Vec<Permission>>,
    ) -> crate::Result<Self> {
        Ok(Self {
            owner_address: address::bs58_addr_to_hex(owner_address)?,
            owner,
            actives,
        })
    }
}

#[async_trait::async_trait]
impl TronTxOperation<MultisigAccountResp> for PermissionUpdateArgs {
    async fn build_raw_transaction(
        &self,
        provider: &Provider,
    ) -> crate::Result<RawTransactionParams> {
        let res: TronTransactionResponse<MultisigAccountResp> = provider
            .do_request("wallet/accountpermissionupdate", Some(self))
            .await?;
        Ok(RawTransactionParams::from(res))
    }

    fn get_to(&self) -> String {
        String::new()
    }

    fn get_value(&self) -> i64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::PermissionTypes;

    #[test]
    pub fn test_default() {
        let permission = PermissionTypes::default().to_hex();

        assert_eq!(
            "7fff1fc0037e0000000000000000000000000000000000000000000000000000",
            permission
        )
    }
}
