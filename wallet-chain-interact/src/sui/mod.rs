mod provider;
pub use provider::*;
mod chain;
pub use chain::*;
mod operations;
pub use operations::*;
pub mod consts;
pub mod error;
pub mod protocol;

pub use sui_json_rpc_types::SuiTransactionBlockData;
pub use sui_json_rpc_types::SuiTransactionBlockEffects;
pub use sui_json_rpc_types::SuiTransactionBlockResponse;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransRespOpt {
    pub show_input: bool,
    pub show_raw_input: bool,
    pub show_effects: bool,
    pub show_events: bool,
    pub show_object_changes: bool,
    pub show_balance_changes: bool,
    pub show_raw_effects: bool,
}

impl TransRespOpt {
    // 解析交易需要用到的参数
    pub fn new_parse() -> Self {
        Self {
            show_input: true,
            show_raw_input: false,
            show_effects: true,
            show_events: false,
            show_object_changes: false,
            show_balance_changes: true,
            show_raw_effects: false,
        }
    }
}

impl Default for TransRespOpt {
    fn default() -> Self {
        Self {
            show_input: true,
            show_raw_input: false,
            show_effects: true,
            show_events: false,
            show_object_changes: false,
            show_balance_changes: false,
            show_raw_effects: false,
        }
    }
}
