use crate::tron::{consts, operations::stake::ResourceType};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct TronAccount {
    pub address: String,
    // unit is trx
    pub balance: i64,
    pub account_resource: AccountResource,
    #[serde(rename = "delegated_frozenV2_balance_for_bandwidth")]
    pub delegated_bandwidth: i64,
    #[serde(rename = "acquired_delegated_frozenV2_balance_for_bandwidth")]
    pub acquired_bandwidth: i64,
    #[serde(rename = "frozenV2")]
    pub frozen_v2: Vec<FrozenV2>,
    #[serde(rename = "unfrozenV2")]
    pub unfreeze_v2: Vec<UnfrozenV2>,
    pub owner_permission: PermissionResp,
    pub active_permission: Vec<PermissionResp>,
    pub votes: Vec<Vote>,
    #[serde(flatten)]
    #[serde(default)]
    extra_fields: std::collections::HashMap<String, serde_json::Value>,
}

impl TronAccount {
    fn now_time(&self) -> i64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_micros() as i64
    }

    // unit is trx
    pub fn frozen_v2_owner(&self, resource_type: &str) -> i64 {
        let sun = self
            .frozen_v2
            .iter()
            .filter(|item| item.types == resource_type)
            .map(|item| item.amount)
            .sum::<i64>();

        sun / consts::TRX_VALUE
    }

    // 可以提取的金额
    pub fn can_withdraw_amount(&self) -> i64 {
        let now = self.now_time();

        self.unfreeze_v2
            .iter()
            .filter(|item| item.unfreeze_expire_time <= now)
            .map(|item| item.unfreeze_amount)
            .sum::<i64>()
            / consts::TRX_VALUE
    }

    // 待提取的能量或者带宽
    pub fn can_withdraw_unfreeze_amount(&self, resource_type: &str) -> i64 {
        let now_time = self.now_time();

        self.unfreeze_v2
            .iter()
            .filter(|item| item.types == resource_type && item.unfreeze_expire_time <= now_time)
            .map(|item| item.unfreeze_amount)
            .sum::<i64>()
            / consts::TRX_VALUE
    }

    // 所有解冻中的能量或者带宽
    pub fn un_freeze_amount(&self, resource_type: &str) -> i64 {
        self.unfreeze_v2
            .iter()
            .filter(|item| item.types == resource_type)
            .map(|item| item.unfreeze_amount)
            .sum::<i64>()
            / consts::TRX_VALUE
    }

    // 有几个带提取的
    pub fn can_withdraw_num(&self) -> i64 {
        let now = self.now_time();

        self.unfreeze_v2
            .iter()
            .filter(|item| item.unfreeze_expire_time <= now)
            .count() as i64
    }

    pub fn balance_to_f64(&self) -> f64 {
        self.balance as f64 / consts::TRX_TO_SUN as f64
    }

    pub fn is_multisig_account(&self) -> bool {
        self.active_permission
            .iter()
            .any(|permission| permission.keys.len() >= 2)
            || self.owner_permission.keys.len() >= 2
    }

    // unit is trx
    pub fn delegate_resource(&self, resource_type: ResourceType) -> i64 {
        let value = match resource_type {
            ResourceType::BANDWIDTH => self.delegated_bandwidth,
            ResourceType::ENERGY => self.account_resource.delegated_energy,
        };

        value / consts::TRX_VALUE
    }

    // unit is trx
    pub fn acquired_resource(&self, resource_type: ResourceType) -> i64 {
        let value = match resource_type {
            ResourceType::BANDWIDTH => self.acquired_bandwidth,
            ResourceType::ENERGY => self.account_resource.acquired_energy,
        };
        value / consts::TRX_VALUE
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct FrozenV2 {
    pub amount: i64,
    #[serde(rename = "type")]
    pub types: String,
    #[serde(flatten)]
    #[serde(default)]
    extra_fields: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct UnfrozenV2 {
    #[serde(rename = "type")]
    pub types: String,
    pub unfreeze_amount: i64,
    pub unfreeze_expire_time: i64,
    #[serde(flatten)]
    #[serde(default)]
    extra_fields: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct AccountResource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_consume_time_for_energy: Option<u64>,
    pub energy_window_size: u64,
    pub energy_window_optimized: bool,
    #[serde(default, rename = "delegated_frozenV2_balance_for_energy")]
    pub delegated_energy: i64,
    #[serde(default, rename = "acquired_delegated_frozenV2_balance_for_energy")]
    pub acquired_energy: i64,
    #[serde(flatten)]
    #[serde(default)]
    extra_fields: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default, rename_all = "PascalCase")]
pub struct AccountResourceDetail {
    #[serde(rename = "freeNetUsed")]
    pub free_net_used: i64,
    #[serde(rename = "freeNetLimit")]
    pub free_net_limit: i64,
    pub net_used: i64,
    pub net_limit: i64,
    pub total_net_limit: i64,
    pub total_net_weight: i64,
    pub energy_used: i64,
    pub energy_limit: i64,
    pub total_energy_limit: i64,
    pub total_energy_weight: i64,
    #[serde(rename = "tronPowerUsed")]
    pub tron_power_used: i64,
    #[serde(rename = "tronPowerLimit")]
    pub tron_power_limit: i64,
}
impl AccountResourceDetail {
    // unit is trx
    pub fn energy_price(&self) -> f64 {
        if self.total_energy_weight == 0 {
            return 0.0;
        }
        self.total_energy_limit as f64 / self.total_energy_weight as f64
    }
    // unit is trx
    pub fn net_price(&self) -> f64 {
        if self.total_net_weight == 0 {
            return 0.0;
        }
        self.total_net_limit as f64 / self.total_net_weight as f64
    }

    pub fn available_bandwidth(&self) -> i64 {
        ((self.net_limit + self.free_net_limit) - (self.net_used + self.free_net_used)).max(0)
    }

    pub fn available_stake_bandwidth(&self) -> i64 {
        (self.net_limit - self.net_used).max(0)
    }

    pub fn available_energy(&self) -> i64 {
        (self.energy_limit - self.energy_used).max(0)
    }

    // value unit is trx
    pub fn resource_value(&self, resource_type: ResourceType, value: i64) -> crate::Result<f64> {
        let price = match resource_type {
            ResourceType::BANDWIDTH => self.net_price(),
            ResourceType::ENERGY => self.energy_price(),
        };

        Ok((price * value as f64 * 100.0).round() / 100.0)
    }
}

/// multi sig account permission
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountPermission<T> {
    owner_address: String,
    owner: T,
    actives: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visible: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Permission {
    #[serde(rename = "type")]
    pub types: i8,
    pub permission_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<String>,
    pub threshold: u8,
    pub keys: Vec<Keys>,
}
#[derive(Serialize, Debug, Deserialize, Default)]
pub struct PermissionResp {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<String>,
    pub permission_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<String>,
    pub threshold: u8,
    pub keys: Vec<Keys>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Keys {
    address: String,
    weight: i8,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Vote {
    pub vote_address: String,
    pub vote_count: i64,
}
