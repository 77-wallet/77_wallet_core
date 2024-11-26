use crate::btc::get_chain;
use wallet_chain_interact::btc::operations;
use wallet_chain_interact::types::ChainPrivateKey;

#[tokio::test]
async fn test_balance() {
    let addr = "address";
    let balance = get_chain().balance(addr, None).await;

    tracing::info!("balance = {:?}", balance);
    assert!(balance.is_ok());

    let balance = wallet_utils::unit::format_to_string(balance.unwrap(), 8).unwrap();
    tracing::info!("balance = {:?}", balance);
}

#[tokio::test]
async fn test_estimate_fee_v1() {
    let instance = get_chain();
    let network = instance.network;
    let to = "address";
    let value = "5";

    // p2pkh
    // let from = "address";
    // let params = operations::transfer::TransferArg::new(from, to, value, "p2pkh", network).unwrap();
    // let tx = instance.estimate_fee_v1(params).await.unwrap();
    // tracing::info!("fee = {:?}", tx);
    // tracing::info!("fee = {:?}", tx.transaction_fee());

    // p2sh-wpkh
    // let from = "address";
    // let params = operations::transfer::TransferArg::new(from, to, value, "p2sh-wpkh", network)
    //     .unwrap()
    //     .with_spend_all(true);
    // let tx = instance.estimate_fee(params).await.unwrap();
    // tracing::info!("fee = {:?}", tx);
    // tracing::info!("fee = {:?}", tx.transaction_fee());

    // p2wpkh
    // let from = "address";
    // let params =
    //     operations::transfer::TransferArg::new(from, to, value, "p2wpkh", network).unwrap();
    // let tx = instance.estimate_fee_v1(params).await.unwrap();
    // tracing::info!("fee = {:?}", tx);
    // tracing::info!("fee = {:?}", tx.transaction_fee());

    // p2tr
    let from = "address";
    let params =
        operations::transfer::TransferArg::new(from, to, value, Some("p2tr".to_string()), network)
            .unwrap()
            .with_spend_all(true);
    let tx = instance.estimate_fee(params).await.unwrap();

    tracing::info!("fee = {:?}", tx);
}

#[tokio::test]
async fn test_transfer_v1() {
    let instance = get_chain();
    let network = instance.network;
    let to = "address";
    let value = "1000";

    // p2pkh
    // let from = "address";
    // let key = ChainPrivateKey::from("private key");
    // let params = operations::transfer::TransferArg::new(from, to, value, "p2pkh", network)
    //     .unwrap()
    //     .with_spend_all(true);
    // let tx = instance.transfer(params, key).await.unwrap();
    // tracing::info!("tx_hash = {:?}", tx);

    // p2sh-wpkh
    // let from = "address";
    // let key = ChainPrivateKey::from("private key");
    // let params = operations::transfer::TransferArg::new(from, to, value, "p2sh-wpkh", network)
    //     .unwrap()
    //     .with_spend_all(true);
    // let tx = instance.transfer(params, key).await.unwrap();
    // tracing::info!("tx_hash = {:?}", tx);

    // p2wpkh
    let from = "address";
    let key = ChainPrivateKey::from("private key");
    let params = operations::transfer::TransferArg::new(
        from,
        to,
        value,
        Some("p2wpkh".to_string()),
        network,
    )
    .unwrap();
    let tx = instance.transfer(params, key).await.unwrap();
    tracing::info!("tx_hash = {:?}", tx);

    // // p2tr
    // let from = "address";
    // let key = ChainPrivateKey::from("private key");
    // let params = operations::transfer::TransferArg::new(from, to, value, "p2tr", network)
    //     .unwrap()
    //     .with_spend_all(true);
    // let tx = instance.transfer(params, key).await.unwrap();
    // tracing::info!("tx_hash = {:?}", tx);
}

#[tokio::test]
async fn test_query_tx() {
    let instance = get_chain();
    let txid = "e891615e3ee99edeb50dd2d1aff1ffe7e90402d052c0f0e802c51a2a40c9a57d";
    let tx_info = instance.query_tx_res(&txid).await;

    tracing::info!("tx_info = {:?}", tx_info);
    assert!(tx_info.is_ok())
}
