// use super::convert_str_to_url;
// use crate::ChainInteractError;
// // use alloy::rpc::types::eth::{BlockNumberOrTag, BlockTransactionsKind, Filter};
// use alloy::{
//     providers::{Provider, ProviderBuilder, RootProvider},
//     pubsub::PubSubFrontend,
//     rpc::{
//         client::WsConnect,
//         types::{BlockNumberOrTag, BlockTransactionsKind, Filter},
//     },
// };
// use futures_util::StreamExt;

// pub struct EthSubscibe {
//     provider: RootProvider<PubSubFrontend>,
// }

// impl EthSubscibe {
//     pub async fn new(ws_url: &str) -> Result<Self, ChainInteractError> {
//         let url = convert_str_to_url(ws_url)?;
//         let ws = WsConnect::new(url);

//         let provider = ProviderBuilder::new()
//             .on_ws(ws)
//             .await
//             .map_err(|e| ChainInteractError::RpcError(e.to_string()))?;

//         Ok(Self { provider })
//     }

//     // 订阅块信息
//     pub async fn subscribe_block(
//         &self,
//         cx: std::sync::Arc<wallet_database::SqliteContext>,
//     ) -> Result<(), ChainInteractError> {
//         let subscription = self
//             .provider
//             .subscribe_blocks()
//             .await
//             .map_err(|e| ChainInteractError::SubError(e.to_string()))?;

//         let mut stream = subscription.into_stream();

//         let tx_handle = Tx {};
//         while let Some(block) = stream.next().await {
//             let tx_block = self
//                 .provider
//                 .get_block_by_hash(block.header.hash.unwrap(), BlockTransactionsKind::Full)
//                 .await;

//             // block 中的所有交易
//             match tx_block {
//                 Ok(tx) => match tx {
//                     Some(block) => {
//                         if let Some(transactions) = block.transactions.as_transactions() {
//                             tx_handle
//                                 .handle_tx(transactions, self.provider.clone(), cx.clone())
//                                 .await;
//                         }
//                     }
//                     None => {
//                         tracing::warn!("none of transaction")
//                     }
//                 },
//                 Err(e) => tracing::error!("get transaction error:{}", e.to_string()),
//             }
//         }

//         Ok(())
//     }

//     // 订阅日志
//     pub async fn subscribe_log(&self) -> Result<(), ChainInteractError> {
//         let events = self.interest_events();

//         println!("events {:?}", events);
//         let filter = Filter::new()
//             .events(self.interest_events())
//             .from_block(BlockNumberOrTag::Latest);

//         // 监听所有的日志，并从中解析出 自己感兴趣的事件
//         let sub = self.provider.subscribe_logs(&filter).await.unwrap();
//         let mut stream = sub.into_stream();

//         while let Some(log) = stream.next().await {
//             println!("filter logs: {log:?}");
//         }

//         Ok(())
//     }

//     fn interest_events(&self) -> Vec<&str> {
//         let mut events = vec![];

//         // transfor 事件
//         events.push("Transfer(address,address,uint256)");

//         // // approve 事件
//         events.push("Approval(address,address,uint)");

//         // // uniwap v3 swap 事件
//         // events.push(keccak256(
//         //     b"Swap(address,address,int256,int256,uint160,uint160,int24)",
//         // ));

//         // // uniwap v2 swap 事件
//         // events.push(keccak256(b"Swap(address,uint,uint,uint,uint,address)"));

//         events
//     }
// }
