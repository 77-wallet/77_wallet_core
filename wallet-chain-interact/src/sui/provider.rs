use serde_json::json;
use sui_sdk::rpc_types::SuiMoveNormalizedModule;
use wallet_transport::{client::RpcClient, types::JsonRpcParams};

pub struct Provider {
    client: RpcClient,
    // _client: sui_sdk::SuiClient,
}

impl Provider {
    pub fn new(rpc_client: RpcClient) -> crate::Result<Self> {
        Ok(Self { client: rpc_client })
    }

    pub async fn balance(&self, addr: &str) -> crate::Result<sui_sdk::rpc_types::Balance> {
        // 将字符串地址转换为 SuiAddress 类型
        let parsed_addr = wallet_utils::address::parse_sui_address(addr)?;
        // self._client
        // .transaction_builder()
        // .
        // self._client
        // .transaction_builder()
        let params = JsonRpcParams::default()
            .method("suix_getBalance")
            .params(json!([parsed_addr.to_string(), "0x2::sui::SUI"])); // 明确指定 SUI 类型

        Ok(self.client.invoke_request(params).await?)
    }

    /// 查询任意代币余额
    pub async fn token_balance(
        &self,
        addr: &str,
        coin_type: &str,
    ) -> crate::Result<sui_sdk::rpc_types::Balance> {
        let parsed_addr = wallet_utils::address::parse_sui_address(addr)?;
        // self._client.coin_read_api()
        // .get_balance(owner, coin_type)

        let params = JsonRpcParams::default()
            .method("suix_getBalance")
            .params(json!([parsed_addr.to_string(), coin_type]));

        Ok(self.client.invoke_request(params).await?)
    }

    /// Gas 费估算（简化版）
    pub async fn get_reference_gas_price(&self) -> crate::Result<u64> {
        let params: JsonRpcParams<()> = JsonRpcParams::default()
            .method("suix_getReferenceGasPrice")
            .no_params();

        let gas: String = self.client.invoke_request(params).await?;
        let gas_price = wallet_utils::parse_func::u64_from_str(&gas)?;
        Ok(gas_price)
    }

    pub async fn calculate_gas_budget(
        &self,
        gas_price: u64,
        gas_budget: u64,
    ) -> crate::Result<u64> {
        // TODO:
        let gas_budget = gas_price * gas_budget;
        Ok(gas_budget)
    }

    pub async fn get_owned_objects(
        &self,
        addr: &str,
        filter: Option<serde_json::Value>,
        cursor: Option<String>,
        limit: Option<u64>,
    ) -> crate::Result<sui_sdk::rpc_types::ObjectsPage> {
        let params = JsonRpcParams::default()
            .method("suix_getOwnedObjects")
            .params(json!([
                addr,
                {
                    "filter": filter,
                    "options": {
                        "showType": true,
                        "showContent": true
                    }
                },
                cursor,
                limit
            ]));
        let res = self.client.invoke_request(params).await?;
        Ok(res)
    }

    pub async fn get_object_by_id(
        &self,
        id: &str,
    ) -> crate::Result<sui_sdk::rpc_types::SuiObjectResponse> {
        let params = JsonRpcParams::default()
            .method("sui_getObject")
            .params(json!([id]));
        let res = self.client.invoke_request(params).await?;
        Ok(res)
    }

    pub(crate) fn sui_coin_filter() -> serde_json::Value {
        json!({
            "MatchAll": [
                { "StructType": "0x2::coin::Coin<0x2::sui::SUI>" }
            ]
        })
    }

    pub async fn get_all_coins_by_owner(
        &self,
        addr: &str,
        coin_type: &str,
    ) -> crate::Result<Vec<sui_sdk::rpc_types::Coin>> {
        let mut cursor: Option<String> = None;
        let mut all_coins = Vec::new();
        loop {
            let params = JsonRpcParams::default()
                .method("suix_getCoins")
                .params(json!([
                    addr,
                    coin_type,
                    cursor,
                    50  // 每页最多50个
                ]));

            let page: sui_sdk::rpc_types::CoinPage = self.client.invoke_request(params).await?;
            all_coins.extend(page.data);

            if page.has_next_page {
                cursor = page.next_cursor;
            } else {
                break;
            }
        }

        Ok(all_coins)
    }

    pub async fn dry_run_transaction(
        &self,
        tx_data: &sui_sdk::types::transaction::TransactionData,
    ) -> crate::Result<sui_sdk::rpc_types::DryRunTransactionBlockResponse> {
        tracing::info!("dry_run_transaction: {:?}", tx_data);
        let tx_data = wallet_utils::serde_func::bcs_to_bytes(tx_data)?;
        let tx_data = wallet_utils::bytes_to_base64(&tx_data);
        let params = JsonRpcParams::default()
            .method("sui_dryRunTransactionBlock")
            .params(json!([tx_data]));
        let res = self.client.invoke_request(params).await?;
        Ok(res)
    }

    pub async fn send_transaction(
        &self,
        tx_bytes_b64: String,
        signatures_b64: Vec<String>,
    ) -> crate::Result<sui_sdk::rpc_types::SuiTransactionBlockResponse> {
        let params = JsonRpcParams::default()
            .method("sui_executeTransactionBlock")
            .params(json!([
                tx_bytes_b64,
                signatures_b64,
                {
                    "showEffects": true,
                    "showEvents": true
                }
            ]));
        let res = self.client.invoke_request(params).await?;
        Ok(res)
    }

    pub async fn get_normalized_move_modules_by_package_id(
        &self,
        package_id: &str,
    ) -> crate::Result<std::collections::BTreeMap<String, SuiMoveNormalizedModule>> {
        let params = JsonRpcParams::default()
            .method("sui_getNormalizedMoveModulesByPackage")
            .params(json!([package_id]));
        let res = self.client.invoke_request(params).await?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::sui::SuiChain;

    use super::*;
    use wallet_utils::init_test_log;

    // Sui DevNet 节点地址
    const DEVNET_URL: &str = "https://fullnode.devnet.sui.io:443";
    const TESTNET_URL: &str = "https://fullnode.testnet.sui.io:443";
    // 测试用地址（Sui DevNet 水龙头示例地址）
    const TEST_ADDRESS: &str = "0x885f29a4f1b4d63822728a1b1811d0278c4e25f27d3754ddd387cd34f9482d0f";
    const TEST_COIN_TYPE: &str = "0x2::sui::SUI";

    fn get_chain() -> SuiChain {
        init_test_log();
        // sui 测试网络
        let rpc = TESTNET_URL.to_string();

        let header = None;
        let client = RpcClient::new(&rpc, header, None).unwrap();
        let provider = Provider::new(client).unwrap();
        let chain_code = wallet_types::chain::chain::ChainCode::Sui;
        let network = wallet_types::chain::network::NetworkKind::Testnet;
        let sui = SuiChain::new(provider, network, chain_code).unwrap();

        sui
    }

    #[tokio::test]
    async fn test_real_sui_balance() {
        let sui = get_chain();

        // 带重试的查询（应对网络波动）
        let balance = sui
            .balance(TEST_ADDRESS, Some(TEST_COIN_TYPE.to_string()))
            .await
            .unwrap();
        println!("balance: {}", balance.total_balance);
        // 验证基础属性
        assert!(!balance.coin_type.is_empty());
        assert!(balance.total_balance > 0);
    }

    #[tokio::test]
    async fn test_estimate_gas() {
        let sui = get_chain();
        let gas = sui.provider.get_reference_gas_price().await.unwrap();
        println!("gas: {}", gas);
        assert!(gas > 0);
    }

    #[tokio::test]
    async fn test_get_owned_objects() {
        let sui = get_chain();
        let filter = Provider::sui_coin_filter();
        let objects = sui
            .provider
            .get_owned_objects(TEST_ADDRESS, Some(filter), None, None)
            .await
            .unwrap();
        println!("objects: {:#?}", objects);
    }

    #[tokio::test]
    async fn test_get_object_by_id() {
        let sui = get_chain();
        let object = sui.provider.get_object_by_id(TEST_ADDRESS).await.unwrap();
        println!("object: {:#?}", object);
    }

    #[tokio::test]
    async fn test_get_normalized_move_modules_by_package_id() {
        let sui = get_chain();
        // let package = ""
        let modules = sui
            .provider
            .get_normalized_move_modules_by_package_id(
                "0xaf9ef585e2efd13321d0a2181e1c0715f9ba28ed052055d33a8b164f6c146a56",
            )
            .await
            .unwrap();
        // println!("modules: {:#?}", modules);

        for (module_name, module) in modules {
            println!("module_name: {}", module_name);
            // println!("module name: {}", module.name);
            println!("module_address: {}", module.address);
            if module_name.eq("coin") {
                println!("module exposed functions: {:#?}", module.exposed_functions);
            }
            // println!("module: {:#?}", module);
        }
    }
}
