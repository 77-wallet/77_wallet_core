use crate::bsc::get_chain;
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

    tracing::info!("balance={rs:?}");
    assert!(rs.is_ok())
}

#[tokio::test]
async fn test_estimate_gas() {
    let instance = get_chain();

    let from = "address";
    let to = "address";
    let value = unit::convert_to_u256("0.01", 18).unwrap();

    let base = operations::EthereumBaseTransaction::new(from, to, value).unwrap();
    let prams = operations::TransferOpt {
        base,
        contract: None,
    };
    let fee = instance.estimate_gas(prams).await.unwrap();

    tracing::info!("fee={fee:?}");
}

#[tokio::test]
async fn test_transfer() {
    let instance = get_chain();

    let from = "address";
    let to = "address";
    let value = unit::convert_to_u256("0.3", 18).unwrap();

    let params = operations::TransferOpt::new(from, to, value, None).unwrap();

    let fee = FeeSetting::default();
    let key = ChainPrivateKey::from("private key");

    let fee = instance.exec_transaction(params, fee, key).await.unwrap();

    tracing::info!("tx_hash={fee:?}");
}

#[tokio::test]
async fn test_query_tx_rs() {
    let instance = get_chain();
    let tx = "0xee344bba865f0e8bd764a14772554b0e87b3f0a104cec2b68ca22f5a894f3227";
    let res = instance.query_tx_res(tx).await;

    tracing::info!("{res:?}");
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_black_address() {
    let chain = get_chain();

    let token = "address";
    let owner = "address";

    let res = chain.black_address(token, owner).await.unwrap();

    tracing::info!("{res:?}");
}
