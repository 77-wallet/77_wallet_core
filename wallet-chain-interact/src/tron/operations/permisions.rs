use super::{
    multisig::{MultisigAccountResp, Permission},
    RawTransactionParams, TronTransactionResponse, TronTxOperation,
};
use crate::tron::Provider;
use wallet_utils::address;

// https://github.com/tronprotocol/java-tron/blob/1f0aa386212feb7817048aeb436779ddecaca534/protocol/src/main/protos/core/Tron.proto#L337
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractType {
    // 创建账号(激活账号)
    AccountCreateContract = 0,
    // TRX转账
    TransferContract = 1,
    // TRC10通证转账
    TransferAssetContract = 2,
    VoteAssetContract = 3,
    // 投票Vote
    VoteWitnessContract = 4,
    // 申请成为超级代表候选人
    WitnessCreateContract = 5,
    // 发行TRC10 资产
    AssetIssueContract = 6,
    // 更新超级代表候选人信息
    WitnessUpdateContract = 8,
    // Participate in TRC10 Issuance
    ParticipateAssetIssueContract = 9,
    // 修改账户名称
    AccountUpdateContract = 10,
    // 质押资产1.0
    FreezeBalanceContract = 11,
    // 解锁资产1.0
    UnfreezeBalanceContract = 12,
    // 提取收益
    WithdrawBalanceContract = 13,
    // Unstake TRC10 ? 锁仓提取
    UnfreezeAssetContract = 14,
    // 更新TRC10 通证参数
    UpdateAssetContract = 15,
    // 发起提议
    ProposalCreateContract = 16,
    // 赞成提议
    ProposalApproveContract = 17,
    // 撤销提议
    ProposalDeleteContract = 18,
    SetAccountIdContract = 19,
    CustomContract = 20,
    // 创建智能合约
    CreateSmartContract = 30,
    // 触发智能合约(TRC20/TRC721转账)
    TriggerSmartContract = 31,
    GetContract = 32,
    // 更新合约参数
    UpdateSettingContract = 33,
    // 创建 Bancor 交易
    ExchangeCreateContract = 41,
    // Bancor 交易注资
    ExchangeInjectContract = 42,
    // Bancor 交易撤资
    ExchangeWithdrawContract = 43,
    // 执行 Bancor 交易
    ExchangeTransactionContract = 44,
    // 更新合约能量限制
    UpdateEnergyLimitContract = 45,
    // 账号权限管理
    AccountPermissionUpdateContract = 46,
    // 清除合约ABI
    ClearABIContract = 48,
    // 更新超级代表佣金比例
    UpdateBrokerageContract = 49,
    ShieldedTransferContract = 51,
    MarketSellAssetContract = 52,
    MarketCancelOrderContract = 53,
    // 质押资产2.0
    FreezeBalanceV2Contract = 54,
    // 解锁资产2.0
    UnfreezeBalanceV2Contract = 55,
    // 提取资产
    WithdrawExpireUnfreezeContract = 56,
    // 代理资源
    DelegateResourceContract = 57,
    // 回收资源
    UnDelegateResourceContract = 58,
    // 全部取消委派
    CancelAllUnfreezeV2Contract = 59,
}

// 权限集合
pub struct PermissionTypes(pub Vec<ContractType>);

impl PermissionTypes {
    pub fn to_hex(&self) -> String {
        let contract_ids = self
            .0
            .iter()
            .map(|i| i.to_i8() as usize)
            .collect::<Vec<usize>>();

        let mut operations = [0u8; 32];
        for &id in &contract_ids {
            operations[id / 8] |= 1 << (id % 8);
        }

        hex::encode(operations)
    }

    pub fn from_hex(hex_str: &str) -> crate::Result<Vec<i8>> {
        let operations = wallet_utils::hex_func::hex_decode(hex_str)?;

        let mut contract_ids = Vec::new();
        for (byte_index, &byte) in operations.iter().enumerate() {
            for bit_index in 0..8 {
                if (byte & (1 << bit_index)) != 0 {
                    contract_ids.push(byte_index * 8 + bit_index);
                }
            }
        }

        let original_structure = contract_ids
            .into_iter()
            .map(|id| id as i8)
            .collect::<Vec<i8>>();

        Ok(original_structure)
    }
}

impl Default for PermissionTypes {
    fn default() -> Self {
        let contract_ids = vec![
            ContractType::AccountCreateContract,           // Activate Account
            ContractType::TransferContract,                // Transfer TRX
            ContractType::TransferAssetContract,           // Transfer TRC10
            ContractType::VoteAssetContract,               // Transfer TRC10
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
            ContractType::SetAccountIdContract,            // Cancel Proposal
            ContractType::CustomContract,                  // Cancel Proposal
            ContractType::CreateSmartContract,             // Create Smart Contract
            ContractType::TriggerSmartContract,            // Trigger Smart Contract
            ContractType::GetContract,                     // Trigger Smart Contract
            ContractType::UpdateSettingContract,           // Update Contract Parameters
            ContractType::ExchangeCreateContract,          // Create Bancor Transaction
            ContractType::ExchangeInjectContract,          // Inject Assets into Bancor Transaction
            ContractType::ExchangeWithdrawContract, // Withdraw Assets from Bancor Transaction
            ContractType::ExchangeTransactionContract, // Execute Bancor Transaction
            ContractType::UpdateEnergyLimitContract, // Update Contract Energy Limit
            ContractType::AccountPermissionUpdateContract, // Update Account Permissions
        ];
        PermissionTypes(contract_ids)
    }
}

impl ContractType {
    pub fn to_i8(&self) -> i8 {
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

    fn get_value(&self) -> f64 {
        0.0
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

    #[test]
    pub fn test_recover() {
        let operations = "7fff1fc0033ec30f000000000000000000000000000000000000000000000000";

        let res = PermissionTypes::from_hex(&operations).unwrap();
        println!("{:?}", res);
        println!("len = {:?}", res.len());
    }
}
