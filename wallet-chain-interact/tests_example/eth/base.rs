use crate::eth::get_chain;
use wallet_chain_interact::{
    eth::{operations, FeeSetting},
    types::ChainPrivateKey,
};
use wallet_utils::unit;

#[tokio::test]
async fn test_get_balance() {
    let instance = get_chain();

    let addr = "address";
    let token: Option<String> = None;
    let rs = instance.balance(addr, token).await;

    let balance = rs.unwrap();
    let balance = wallet_utils::unit::format_to_string(balance, 18).unwrap();

    tracing::info!("balance={balance:?}");
}

#[tokio::test]
async fn test_estimate_gas() {
    let instance = get_chain();

    let from = "address";
    let to = "address";
    let value = unit::convert_to_u256("0.01", 18).unwrap();

    let params = operations::TransferOpt::new(from, to, value, None).unwrap();

    let fee = instance.estimate_gas(params).await.unwrap();
    tracing::info!("fee={fee:?}");
}

#[tokio::test]
async fn test_transfer() {
    let instance = get_chain();

    let from = "address";
    let to = "address";
    let value = unit::convert_to_u256("9", 18).unwrap();

    let params = operations::TransferOpt::new(from, to, value, None).unwrap();

    let fee = FeeSetting::default();
    let key = ChainPrivateKey::from("1");

    let fee = instance.exec_transaction(params, fee, key).await.unwrap();

    tracing::info!("fee={fee:?}");
}

#[tokio::test]
async fn test_query_tx_rs() {
    let instance = get_chain();
    let tx = "0xe10d96ec2a982bd062abe40d347ef4e7b92a6ebc341afbba0b7c13f79241b746";
    let res = instance.query_tx_res(tx).await;

    tracing::info!("{res:?}");
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_chain_id() {
    let instance = get_chain();
    let res = instance.provider.chain_id().await.unwrap();
    tracing::info!("{res:?}");
}

#[tokio::test]
async fn test_default_fee() {
    let instance = get_chain();
    let res = instance.provider.get_default_fee().await.unwrap();
    tracing::info!("{res:?}");
}

#[tokio::test]
async fn test_black_address() {
    let chain = get_chain();

    let token = "address";
    let owner = "address";

    let res = chain.black_address(token, owner).await.unwrap();

    tracing::info!("{res:?}");
}
