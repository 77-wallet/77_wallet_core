// use alloy::providers::Provider;
// use alloy::rpc::types::Transaction;
// use alloy::{providers::RootProvider, pubsub::PubSubFrontend};
// use wallet_database::sqlite::service::bill::TransactionVo;

// use crate::convert_eth_address;
// pub struct Tx;

// impl Tx {
//     pub async fn handle_tx(
//         &self,
//         transcations: &[Transaction],
//         provider: RootProvider<PubSubFrontend>,
//         cx: std::sync::Arc<wallet_database::SqliteContext>,
//     ) {
//         // 找到与自己相关的交易，，，如果存在， 更新余额，插入交易表。
//         let own_transcations = self.own_transaction_lists(transcations, cx).await;
//         if own_transcations.is_empty() {
//             return;
//         }

//         for transaction in own_transcations {
//             // 获取当前账户的余额
//             let address = convert_eth_address(&transaction.address).unwrap();
//             let balance = provider.get_balance(address).await;

//             match balance {
//                 Ok(_balance) => {
//                     // let rs = wallet_database::sqlite::service::bill::update_bill_with_balnce(
//                     //     transaction,
//                     //     balance.to_string(),
//                     // )
//                     // .await;
//                     // if let Err(e) = rs {
//                     //     tracing::error!("update bill error :{}", e);
//                     // }
//                 }
//                 Err(e) => tracing::error!("get balance error{}", e),
//             }
//         }
//     }

//     // 属于自己的交易列表
//     pub async fn own_transaction_lists(
//         &self,
//         transcations: &[Transaction],
//         cx: std::sync::Arc<wallet_database::SqliteContext>,
//     ) -> Vec<TransactionVo> {
//         //  是否考虑根据不同链获取不同的账户
//         let accounts = cx.account_list().await;

//         let mut own_tx = vec![];
//         match accounts {
//             Ok(accounts) => {
//                 for tx in transcations {
//                     for account in &accounts {
//                         let from = tx.from.to_string();
//                         let to = tx.to.unwrap_or_default().to_string();
//                         if from == account.address || to == account.address {
//                             let tx_vo = TransactionVo::new(tx.clone(), account);
//                             own_tx.push(tx_vo)
//                         }
//                     }
//                 }
//             }
//             Err(e) => tracing::error!("get account from sqlite error: {}", e),
//         }
//         own_tx
//     }
// }
