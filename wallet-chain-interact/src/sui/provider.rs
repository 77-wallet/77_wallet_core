use serde_json::json;
use wallet_transport::{client::RpcClient, types::JsonRpcParams};

pub struct Provider {
    client: RpcClient,
    // _client: sui_sdk::SuiClient,
}

impl Provider {
    pub fn new(rpc_client: RpcClient) -> crate::Result<Self> {
        Ok(Self { client: rpc_client })
    }

    /// 统一地址解析方法
    fn parse_address(addr: &str) -> crate::Result<sui_sdk::types::base_types::SuiAddress> {
        Ok(addr
            .parse()
            .map_err(|e| crate::ParseErr::AddressPraseErr(format!("err:{} address:{}", e, addr)))?)
    }
    pub async fn balance(&self, addr: &str) -> crate::Result<sui_sdk::rpc_types::Balance> {
        // 将字符串地址转换为 SuiAddress 类型
        let parsed_addr = Self::parse_address(addr)?;

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
        let parsed_addr = Self::parse_address(addr)?;
        // self._client.coin_read_api()
        // .get_balance(owner, coin_type)

        let params = JsonRpcParams::default()
            .method("suix_getBalance")
            .params(json!([parsed_addr.to_string(), coin_type]));

        Ok(self.client.invoke_request(params).await?)
    }

    /// Gas 费估算（简化版）
    pub async fn estimate_gas(&self) -> crate::Result<u64> {
        let params: JsonRpcParams<()> = JsonRpcParams::default()
            .method("suix_getReferenceGasPrice")
            .no_params();

        let response: serde_json::Value = self.client.invoke_request(params).await?;

        response["result"]
            .as_u64()
            .ok_or_else(|| crate::Error::RpcError("Invalid gas price response".into()))
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
}
