use super::{TransRespOpt, protocol::CheckpointResult};
use serde_json::json;
use sui_json_rpc_types::{
    Balance, Coin, CoinPage, DevInspectResults, DryRunTransactionBlockResponse, ObjectsPage,
    SuiCoinMetadata, SuiMoveNormalizedModule, SuiObjectResponse, SuiTransactionBlockResponse,
};
use sui_types::transaction::{ProgrammableTransaction, TransactionData, TransactionKind};
use wallet_transport::{client::RpcClient, types::JsonRpcParams};

pub struct Provider {
    client: RpcClient,
}

impl Provider {
    pub fn new(rpc_client: RpcClient) -> Self {
        Self { client: rpc_client }
    }

    pub async fn balance(&self, addr: &str, coin_type: &str) -> crate::Result<Balance> {
        // 将字符串地址转换为 SuiAddress 类型
        let parsed_addr = wallet_utils::address::parse_sui_address(addr)?;
        let params = JsonRpcParams::default()
            .method("suix_getBalance")
            .params(json!([parsed_addr.to_string(), coin_type])); // 明确指定 SUI 类型

        Ok(self.client.invoke_request(params).await?)
    }

    pub async fn latest_block(&self) -> crate::Result<String> {
        let params: JsonRpcParams<()> = JsonRpcParams::default()
            .method("sui_getLatestCheckpointSequenceNumber")
            .no_params();

        Ok(self.client.invoke_request(params).await?)
    }

    pub async fn query_tx_info(
        &self,
        digest: &str,
        opt: TransRespOpt,
    ) -> crate::Result<SuiTransactionBlockResponse> {
        let params = JsonRpcParams::default()
            .method("sui_getTransactionBlock")
            .params(json!([digest, opt]));

        Ok(self.client.invoke_request(params).await?)
    }

    pub async fn get_coin_metadata(&self, coin_type: &str) -> crate::Result<SuiCoinMetadata> {
        let params = JsonRpcParams::default()
            .method("suix_getCoinMetadata")
            .params(json!([coin_type]));

        Ok(self.client.invoke_request(params).await?)
    }

    pub async fn get_reference_gas_price(&self) -> crate::Result<u64> {
        let params: JsonRpcParams<()> = JsonRpcParams::default()
            .method("suix_getReferenceGasPrice")
            .no_params();

        let gas: String = self.client.invoke_request(params).await?;
        let gas_price = wallet_utils::parse_func::u64_from_str(&gas)?;
        Ok(gas_price)
    }

    pub async fn get_owned_objects(
        &self,
        addr: &str,
        filter: Option<serde_json::Value>,
        cursor: Option<String>,
        limit: Option<u64>,
    ) -> crate::Result<ObjectsPage> {
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

    pub async fn get_object_by_id(&self, id: &str) -> crate::Result<SuiObjectResponse> {
        let params = JsonRpcParams::default()
            .method("sui_getObject")
            .params(json!([id]));
        let res = self.client.invoke_request(params).await?;
        Ok(res)
    }

    pub async fn get_all_coins_by_owner(
        &self,
        addr: &str,
        coin_type: &str,
    ) -> crate::Result<Vec<Coin>> {
        let mut cursor: Option<String> = None;
        let mut all_coins = Vec::new();
        loop {
            let params = JsonRpcParams::default()
                .method("suix_getCoins")
                .params(json!([
                    addr, coin_type, cursor, 50 // 每页最多50个
                ]));

            let page: CoinPage = self.client.invoke_request(params).await?;
            all_coins.extend(page.data);

            if page.has_next_page {
                cursor = page.next_cursor;
            } else {
                break;
            }
        }

        Ok(all_coins)
    }

    pub async fn dev_inspect_transaction(
        &self,
        sender: &str,
        tx: ProgrammableTransaction,
        gas_price: u64,
    ) -> crate::Result<DevInspectResults> {
        let tx = TransactionKind::programmable(tx);

        let boc_str = wallet_utils::serde_func::bcs_to_bytes(&tx)?;
        let boc_str = wallet_utils::bytes_to_base64(&boc_str);

        let params = JsonRpcParams::default()
            .method("sui_devInspectTransactionBlock")
            .params(json!([sender, boc_str, gas_price.to_string()]));

        Ok(self.client.invoke_request(params).await?)
    }

    pub async fn dry_run_transaction(
        &self,
        tx_data: &TransactionData,
    ) -> crate::Result<DryRunTransactionBlockResponse> {
        let tx_data = wallet_utils::serde_func::bcs_to_bytes(tx_data)?;
        let tx_data = wallet_utils::bytes_to_base64(&tx_data);
        let params = JsonRpcParams::default()
            .method("sui_dryRunTransactionBlock")
            .params(json!([tx_data]));

        Ok(self.client.invoke_request(params).await?)
    }

    pub async fn get_check_point(&self, check_point: &str) -> crate::Result<CheckpointResult> {
        let params = JsonRpcParams::default()
            .method("sui_getCheckpoint")
            .params(json!([check_point]));

        Ok(self.client.invoke_request(params).await?)
    }

    pub async fn get_multi_trans(
        &self,
        digests: &[String],
        opt: TransRespOpt,
    ) -> crate::Result<Vec<SuiTransactionBlockResponse>> {
        let params = JsonRpcParams::default()
            .method("sui_multiGetTransactionBlocks")
            .params(json!([digests, opt]));

        Ok(self.client.invoke_request(params).await?)
    }

    pub async fn send_transaction(
        &self,
        tx_bytes_b64: String,
        signatures_b64: Vec<String>,
    ) -> crate::Result<SuiTransactionBlockResponse> {
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

        Ok(self.client.invoke_request(params).await?)
    }

    pub async fn get_normalized_move_modules_by_package_id(
        &self,
        package_id: &str,
    ) -> crate::Result<std::collections::BTreeMap<String, SuiMoveNormalizedModule>> {
        let params = JsonRpcParams::default()
            .method("sui_getNormalizedMoveModulesByPackage")
            .params(json!([package_id]));

        Ok(self.client.invoke_request(params).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sui::SuiChain;
    use wallet_utils::init_test_log;

    // Sui DevNet 节点地址
    // const DEVNET_URL: &str = "https://fullnode.devnet.sui.io:443";
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
        let provider = Provider::new(client);
        let sui = SuiChain::new(provider).unwrap();
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
        println!("balance: {}", balance);
    }

    #[tokio::test]
    async fn test_estimate_gas_price() {
        let sui = get_chain();
        let gas = sui.provider.get_reference_gas_price().await.unwrap();
        println!("gas: {}", gas);
        assert!(gas > 0);
    }

    #[tokio::test]
    async fn test_get_object_by_id() {
        let sui = get_chain();
        let object = sui.provider.get_object_by_id(TEST_ADDRESS).await.unwrap();
        println!("object: {:#?}", object);
    }

    #[tokio::test]
    async fn test_latest_block() {
        let sui = get_chain();
        let block = sui.provider.latest_block().await.unwrap();
        println!("block: {:#?}", block);
    }

    #[tokio::test]
    async fn test_get_normalized_move_modules_by_package_id() {
        let sui = get_chain();
        // let package = ""
        let modules = sui
            .provider
            .get_normalized_move_modules_by_package_id(
                "0x8190b041122eb492bf63cb464476bd68c6b7e570a4079645a8b28732b6197a82",
            )
            .await
            .unwrap();
        // println!("modules: {:#?}", modules);

        for (module_name, module) in modules {
            println!("module_name: {}", module_name);
            println!("module_address: {}", module.address);
            println!("module_structs: {:#?}", module.structs);
            println!("module enums: {:#?}", module.enums);
        }
    }

    #[tokio::test]
    async fn test_query_tx_info() {
        let sui = get_chain();

        let opt = TransRespOpt::default();

        let tx_info = sui
            .provider
            .query_tx_info("GdyEZutEWFwJuNj2N9aXB5K2L5L3WsvwDSkxBsCb7y2n", opt)
            .await
            .unwrap();
        println!("tx_info: {:#?}", tx_info);
    }
}
