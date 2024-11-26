use crate::sol::get_chain;
use wallet_chain_interact::sol::operations::{self, SolInstructionOperation};

#[tokio::test]
async fn test_multi_address() {
    let res = operations::multisig::account::MultisigAccountOpt::multisig_address().unwrap();

    println!("multisig address {res:?}")
}

fn get_owners() -> Vec<String> {
    vec![
        "address".to_string(),
        "address".to_string(),
        "address".to_string(),
    ]
}

#[tokio::test]
async fn test_deploy_fee() {
    let instance = get_chain();
    let from = "address";
    let salt = "salt".to_string();

    let params = operations::multisig::account::MultisigAccountOpt::new(
        &from,
        2,
        get_owners(),
        salt,
        instance.get_provider(),
    )
    .unwrap();
    let instructions = params.instructions().await.unwrap();

    let rs = instance
        .estimate_fee_v1(&instructions, &params)
        .await
        .unwrap();

    tracing::info!("deploy account {:?}", rs.transaction_fee());
}

#[tokio::test]
async fn test_deploy_multisig() {
    let instance = get_chain();
    let from = "address";

    let create_key = solana_sdk::signature::Keypair::new();
    let salt = create_key.to_base58_string();

    let params = operations::multisig::account::MultisigAccountOpt::new(
        &from,
        2,
        get_owners(),
        salt,
        instance.get_provider(),
    )
    .unwrap();
    let key = "private key";

    let instructions = params.instructions().await.unwrap();
    let res = instance
        .exec_transaction(params, key.into(), None, instructions, 0)
        .await
        .unwrap();

    tracing::info!("deploy account {}", res);
}

#[tokio::test]
async fn test_create_multi_transfer() {
    let chain = get_chain();

    let from = "address";
    let to = "address";
    let value = "0.001";
    let decimal = 9;
    let token = None;

    let params = operations::transfer::TransferOpt::new(
        from,
        to,
        value,
        token,
        decimal,
        chain.get_provider(),
    )
    .unwrap();

    let creator = "address";
    let multisig_pda = "pad account";
    let multisig =
        operations::multisig::transfer::BuildTransactionOpt::new(multisig_pda, 3, creator, params)
            .unwrap();

    // 获取交易参数
    let args = multisig.build_transaction_arg().await.unwrap();
    // 交易指令
    let instructions = multisig.instructions(&args).await.unwrap();

    // 预估手续费
    let base_fee = chain
        .estimate_fee_v1(&instructions, &multisig)
        .await
        .unwrap();

    let _extra = multisig
        .create_transaction_fee(&args.transaction_message, base_fee)
        .await
        .unwrap();
    // tracing::info!("fee {:?}", _extra.transaction_fee());
    let pda = multisig.multisig_pda.clone();
    let key = "private key";

    let c = chain
        .exec_transaction(multisig, key.into(), None, instructions, 0)
        .await
        .unwrap();
    let resp = args.get_raw_data(pda, c).unwrap();

    tracing::info!("tx hash ={:?}", resp);
}

#[tokio::test]
async fn test1_sign_transaction() {
    let signer = "address";
    let key = "private key";
    let data = "raw data";
    let params =
        operations::multisig::transfer::SignTransactionOpt::new(signer, data.to_string()).unwrap();

    let instructions = params.instructions().await.unwrap();
    let res = get_chain()
        .sign_with_res(instructions, params, key.into())
        .await
        .unwrap();

    tracing::info!("sing res {:?}", res);
}

#[tokio::test]
async fn test2_sign_transaction() {
    let signer = "address";
    let key = "private key";
    let data = "raw data";
    let params =
        operations::multisig::transfer::SignTransactionOpt::new(signer, data.to_string()).unwrap();

    let instructions = params.instructions().await.unwrap();
    let res = get_chain()
        .sign_with_res(instructions, params, key.into())
        .await
        .unwrap();

    tracing::info!("sing res {:?}", res);
}

#[tokio::test]
async fn test_exec_multi_transaction() {
    let executor = "address";
    let keypair = "key pair";
    let raw_data = "raw data";

    let params =
        operations::multisig::transfer::ExecMultisigOpt::new(executor, raw_data.to_string())
            .unwrap();

    let instructions = params.instructions().await.unwrap();

    let res = get_chain()
        .exec_transaction(params, keypair.into(), None, instructions, 0)
        .await
        .unwrap();

    tracing::info!("get transaction hash = {:?}", res);
}
