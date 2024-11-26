use crate::btc::get_chain;
use wallet_chain_interact::btc::operations;
use wallet_types::valueobject::AddressPubkey;

fn get_owners() -> Vec<AddressPubkey> {
    vec![
        AddressPubkey {
            address: "".to_string(),
            pubkey: "pubkey".to_string(),
        },
        AddressPubkey {
            address: "".to_string(),
            pubkey: "pubkey".to_string(),
        },
        AddressPubkey {
            address: "".to_string(),
            pubkey: "pubkey".to_string(),
        },
    ]
}

#[tokio::test]
async fn test_multi_address() {
    let instance = get_chain();

    let params = operations::multisig::MultisigAccountOpt::new(2, get_owners(), "p2sh").unwrap();
    let res = instance.multisig_address(params).await.unwrap();
    tracing::info!("p2sh address = {:#?}", res);
    assert_eq!("address", res.multisig_address);

    // let params =
    //     operations::multisig::MultisigAccountOpt::new(2, get_owners(), "p2sh-wsh").unwrap();
    // let res = instance.multisig_address(params).await.unwrap();
    // tracing::info!("p2sh-wsh address = {:?}", res);
    // assert_eq!("address", res.multisig_address);

    // let params = operations::multisig::MultisigAccountOpt::new(2, get_owners(), "p2wsh").unwrap();
    // let res = instance.multisig_address(params).await.unwrap();
    // tracing::info!("p2wsh address = {:?}", res);
    // assert_eq!(
    //     "address",
    //     res.multisig_address
    // );

    // let params = operations::multisig::MultisigAccountOpt::new(2, get_owners(), "p2tr-sh").unwrap();
    // let res = instance.multisig_address(params).await.unwrap();
    // tracing::info!("p2tr-sh address = {:#?}", res);
}

#[tokio::test]
async fn test_create_multisig_transfer() {
    let instance = get_chain();
    let network = instance.network;
    // p2sh
    let from = "address";
    // p2wsh
    // let from = "address";
    // p2sh-wsh
    // let from = "address";
    // p2tr
    // let from = "address";

    let to = "address";
    let value = "0.1";
    let address_type = "p2sh";

    let prarms = operations::transfer::TransferArg::new(
        from,
        to,
        value,
        Some(address_type.to_string()),
        network,
    )
    .unwrap();

    let res = instance.build_multisig_tx(prarms).await.unwrap();
    tracing::info!("get transaction hash = {:?}", res);
}

#[tokio::test]
async fn test1_sign_transaction() {
    let instance = get_chain();
    let from = "".to_string();
    let value = "".to_string();

    let script_hex = "5221022b1c8becf58ce0a7db2eaf5666f295c7c8343077e09a0b2666eb51f1cbc084462102923ae9757390d24e39439d7bd337f1cbfdce38048ee004afd88e1cea099719bf21024a9c26d9c395129c8c097a7b255568410ea9d4c093b229b8c96a25f3435bdc1453ae";
    let raw_data = "2300000000000000324d766633444d7957693261623370615476704c6a327347653179754b47434d444a4501000000000000004200000000000000356235383337613935393635313932313534643935656162363265366439656664326531396264333036376337333033303861386263306561633335356635312d314000000000000000356235383337613935393635313932313534643935656162363265366439656664326531396264333036376337333033303861386263306561633335356635310100000028c9fa02000000000600000001e400000000000000303230303030303030313531356633356163306562636138303830333733376330366433396265316432656664396536363261623565643935343231313936353539613933373538356230313030303030303030666466666666666630323830393639383030303030303030303031363030313439316132643534313363613331663232353832323834376261643165363935366230386239376165313032613632303230303030303030303137613931343235363935646337613031323433376535303930336633656466346161653733663835356537666638373030303030303030";

    let address_type = "p2sh";
    let params = operations::multisig::MultisigTransactionOpt::new(
        from,
        value,
        script_hex,
        raw_data,
        address_type,
    )
    .unwrap();
    let key = "private key";

    let res = instance.sign_multisig_tx(params, key.into()).await.unwrap();

    tracing::info!("sign res == {res:?}");
}

#[tokio::test]
async fn test2_sign_transaction() {
    let instance = get_chain();

    let from = "".to_string();
    let value = "".to_string();

    let script_hex = "5221022b1c8becf58ce0a7db2eaf5666f295c7c8343077e09a0b2666eb51f1cbc084462102923ae9757390d24e39439d7bd337f1cbfdce38048ee004afd88e1cea099719bf21024a9c26d9c395129c8c097a7b255568410ea9d4c093b229b8c96a25f3435bdc1453ae";
    let raw_data = "2300000000000000324d766633444d7957693261623370615476704c6a327347653179754b47434d444a4501000000000000004200000000000000303962313161373432313232653066373030663064313538653936353562313233323962643138363335616535623165373737386466646139373963353963352d314000000000000000303962313161373432313232653066373030663064313538653936353562313233323962643138363335616535623165373737386466646139373963353963350100000040c2f505000000000600000001e400000000000000303230303030303030316335353939633937646164663738373731653562616533353836643139623332313235623635653935386431663030306637653032323231373431616231303930313030303030303030666466666666666630323830663066613032303030303030303031363030313439316132643534313363613331663232353832323834376261643165363935366230386239376165323863396661303230303030303030303137613931343235363935646337613031323433376535303930336633656466346161653733663835356537666638373030303030303030";

    let address_type = "p2sh";
    let params = operations::multisig::MultisigTransactionOpt::new(
        from,
        value,
        script_hex,
        raw_data,
        address_type,
    )
    .unwrap();
    let key = "cT9bnaLgcNHRx7FwnxVwLtk87XAvrukv4ppjUdFPeoTJ1hYzjqta";

    let res = instance.sign_multisig_tx(params, key.into()).await.unwrap();

    tracing::info!("sign res == {res:?}");
}

#[tokio::test]
async fn test_exec_multi_transaction() {
    let instance = get_chain();

    let from = "".to_string();
    let value = "".to_string();
    let script_hex = "5221022b1c8becf58ce0a7db2eaf5666f295c7c8343077e09a0b2666eb51f1cbc084462102923ae9757390d24e39439d7bd337f1cbfdce38048ee004afd88e1cea099719bf21024a9c26d9c395129c8c097a7b255568410ea9d4c093b229b8c96a25f3435bdc1453ae";
    let raw_data = "2300000000000000324d766633444d7957693261623370615476704c6a327347653179754b47434d444a4501000000000000004200000000000000303962313161373432313232653066373030663064313538653936353562313233323962643138363335616535623165373737386466646139373963353963352d314000000000000000303962313161373432313232653066373030663064313538653936353562313233323962643138363335616535623165373737386466646139373963353963350100000040c2f505000000000600000001e400000000000000303230303030303030316335353939633937646164663738373731653562616533353836643139623332313235623635653935386431663030306637653032323231373431616231303930313030303030303030666466666666666630323830663066613032303030303030303031363030313439316132643534313363613331663232353832323834376261643165363935366230386239376165323863396661303230303030303030303137613931343235363935646337613031323433376535303930336633656466346161653733663835356537666638373030303030303030";

    let address_type = "p2sh";
    let params = operations::multisig::MultisigTransactionOpt::new(
        from,
        value,
        script_hex,
        raw_data,
        address_type,
    )
    .unwrap();
    let sign = vec!["sign1".to_string(), "sign2".to_string()];

    let res = instance
        .exec_multisig_tx(params, sign, "".to_string())
        .await;
    tracing::info!("get transaction hash = {:?}", res);
    assert!(res.is_ok())
}

#[tokio::test]
async fn test_p2sh() {
    let chain = get_chain();
    let network = chain.network;

    // build transaction
    let from = "address";
    let to = "address";
    let value = "0.1";
    let address_type = "p2sh";

    // build transaction
    let prarms = operations::transfer::TransferArg::new(
        from,
        to,
        value,
        Some(address_type.to_string()),
        network,
    )
    .unwrap();
    let res = chain.build_multisig_tx(prarms).await.unwrap();

    let script_hex = "5221022b1c8becf58ce0a7db2eaf5666f295c7c8343077e09a0b2666eb51f1cbc084462102923ae9757390d24e39439d7bd337f1cbfdce38048ee004afd88e1cea099719bf21024a9c26d9c395129c8c097a7b255568410ea9d4c093b229b8c96a25f3435bdc1453ae";

    let mut signatures = vec![];

    // sign 2
    let params = operations::multisig::MultisigTransactionOpt::new(
        from.to_string(),
        value.to_string(),
        script_hex,
        &res.raw_data,
        address_type,
    )
    .unwrap();
    let key = "private key";
    let sign2 = chain.sign_multisig_tx(params, key.into()).await.unwrap();
    signatures.push(sign2.signature);

    // sing 1
    let params = operations::multisig::MultisigTransactionOpt::new(
        from.to_string(),
        value.to_string(),
        script_hex,
        &res.raw_data,
        address_type,
    )
    .unwrap();
    let key = "private key";
    let sign1 = chain.sign_multisig_tx(params, key.into()).await.unwrap();
    signatures.push(sign1.signature);

    let params = operations::multisig::MultisigTransactionOpt::new(
        from.to_string(),
        value.to_string(),
        script_hex,
        &res.raw_data,
        address_type,
    )
    .unwrap();

    // exec
    let res = chain
        .exec_multisig_tx(params, signatures, "".to_string())
        .await
        .unwrap();
    tracing::info!("get transaction hash = {:?}", res);
}

#[tokio::test]
async fn test_p2sh_wsh() {
    let chain = get_chain();
    let network = chain.network;

    // build transaction
    let from = "address";
    let to = "address";
    let value = "0.3";
    let address_type = "p2sh-wsh";

    // build transaction
    let prarms = operations::transfer::TransferArg::new(
        from,
        to,
        value,
        Some(address_type.to_string()),
        network,
    )
    .unwrap();
    let res = chain.build_multisig_tx(prarms).await.unwrap();

    let script_hex = "5221022b1c8becf58ce0a7db2eaf5666f295c7c8343077e09a0b2666eb51f1cbc084462102923ae9757390d24e39439d7bd337f1cbfdce38048ee004afd88e1cea099719bf21024a9c26d9c395129c8c097a7b255568410ea9d4c093b229b8c96a25f3435bdc1453ae";

    let mut signatures = vec![];

    // sing 1
    let params = operations::multisig::MultisigTransactionOpt::new(
        from.to_string(),
        value.to_string(),
        script_hex,
        &res.raw_data,
        address_type,
    )
    .unwrap();
    let key = "private key";
    let sign1 = chain.sign_multisig_tx(params, key.into()).await.unwrap();
    signatures.push(sign1.signature);

    // sign 2
    let params = operations::multisig::MultisigTransactionOpt::new(
        from.to_string(),
        value.to_string(),
        script_hex,
        &res.raw_data,
        address_type,
    )
    .unwrap();
    let key = "private key";
    let sign2 = chain.sign_multisig_tx(params, key.into()).await.unwrap();
    signatures.push(sign2.signature);

    let params = operations::multisig::MultisigTransactionOpt::new(
        from.to_string(),
        value.to_string(),
        script_hex,
        &res.raw_data,
        address_type,
    )
    .unwrap();

    // exec
    let res = chain
        .exec_multisig_tx(params, signatures, "".to_string())
        .await
        .unwrap();
    tracing::info!("get transaction hash = {:?}", res);
}

#[tokio::test]
async fn test_p2wsh() {
    let chain = get_chain();
    let network = chain.network;

    // build transaction
    let from = "address";
    let to = "address";
    let value = "0.3";
    let address_type = "p2wsh";

    // build transaction
    let prarms = operations::transfer::TransferArg::new(
        from,
        to,
        value,
        Some(address_type.to_string()),
        network,
    )
    .unwrap();
    let res = chain.build_multisig_tx(prarms).await.unwrap();

    let script_hex = "5221022b1c8becf58ce0a7db2eaf5666f295c7c8343077e09a0b2666eb51f1cbc084462102923ae9757390d24e39439d7bd337f1cbfdce38048ee004afd88e1cea099719bf21024a9c26d9c395129c8c097a7b255568410ea9d4c093b229b8c96a25f3435bdc1453ae";

    let mut signatures = vec![];

    // sing 1
    let params = operations::multisig::MultisigTransactionOpt::new(
        from.to_string(),
        value.to_string(),
        script_hex,
        &res.raw_data,
        address_type,
    )
    .unwrap();
    let key = "private key";
    let sign1 = chain.sign_multisig_tx(params, key.into()).await.unwrap();
    signatures.push(sign1.signature);

    // sign 2
    let params = operations::multisig::MultisigTransactionOpt::new(
        from.to_string(),
        value.to_string(),
        script_hex,
        &res.raw_data,
        address_type,
    )
    .unwrap();
    let key = "private key";
    let sign2 = chain.sign_multisig_tx(params, key.into()).await.unwrap();
    signatures.push(sign2.signature);

    let params = operations::multisig::MultisigTransactionOpt::new(
        from.to_string(),
        value.to_string(),
        script_hex,
        &res.raw_data,
        address_type,
    )
    .unwrap();

    // exec
    let res = chain
        .exec_multisig_tx(params, signatures, "".to_string())
        .await
        .unwrap();
    tracing::info!("get transaction hash = {:?}", res);
}

#[tokio::test]
async fn test_p2tr_sh() {
    let chain = get_chain();
    let network = chain.network;

    // build transaction
    let from = "address";
    let to = "address";
    let value = "0.3";
    let address_type = "p2tr-sh";

    // build transaction
    let prarms = operations::transfer::TransferArg::new(
        from,
        to,
        value,
        Some(address_type.to_string()),
        network,
    )
    .unwrap();
    let res = chain.build_multisig_tx(prarms).await.unwrap();

    let script_hex = "202b1c8becf58ce0a7db2eaf5666f295c7c8343077e09a0b2666eb51f1cbc08446ac20923ae9757390d24e39439d7bd337f1cbfdce38048ee004afd88e1cea099719bfba204a9c26d9c395129c8c097a7b255568410ea9d4c093b229b8c96a25f3435bdc14ba52a2";

    let mut signatures = vec![];

    signatures.push("".to_string());

    // sign 2
    let params = operations::multisig::MultisigTransactionOpt::new(
        from.to_string(),
        value.to_string(),
        script_hex,
        &res.raw_data,
        address_type,
    )
    .unwrap();
    let key = "private key";
    let sign2 = chain.sign_multisig_tx(params, key.into()).await.unwrap();
    signatures.push(sign2.signature);

    // sing 1
    let params = operations::multisig::MultisigTransactionOpt::new(
        from.to_string(),
        value.to_string(),
        script_hex,
        &res.raw_data,
        address_type,
    )
    .unwrap();
    let key = "private key";
    let sign1 = chain.sign_multisig_tx(params, key.into()).await.unwrap();
    signatures.push(sign1.signature);

    let params = operations::multisig::MultisigTransactionOpt::new(
        from.to_string(),
        value.to_string(),
        script_hex,
        &res.raw_data,
        address_type,
    )
    .unwrap();

    // exec
    let res = chain
        .exec_multisig_tx(
            params,
            signatures,
            "552cdba7c2228a6102b7741b2006f16841ff149163f6c4ce5719e27a25c9d8ab".to_string(),
        )
        .await
        .unwrap();
    tracing::info!("get transaction hash = {:?}", res);
}
