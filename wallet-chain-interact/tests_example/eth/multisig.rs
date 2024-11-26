use crate::eth::get_chain;
use wallet_chain_interact::{
    eth::{operations, FeeSetting},
    types::ChainPrivateKey,
};
use wallet_utils::unit;

// 多签账户合约
const MULTISIG_ACCOUNT_ADDR: &str = "";

fn get_owners() -> Vec<String> {
    vec!["address".to_string(), "address".to_string()]
}

#[tokio::test]
async fn test_multisig_address() {
    let instance = get_chain();

    let from = "address";
    let params = operations::MultisigAccountOpt::new(from, 2)
        .unwrap()
        .with_nonce()
        .with_owners(get_owners())
        .unwrap();

    let res = instance.multisig_account(params).await.unwrap();
    tracing::info!("result of create multisig account  = {:?}", res);
}

#[tokio::test]
async fn test_deploy_ccount() {
    let instance = get_chain();

    let from = "address";
    let params = operations::MultisigAccountOpt::new(from, 2)
        .unwrap()
        .with_nonce()
        .with_owners(get_owners())
        .unwrap();

    let fee_setting = FeeSetting::default();
    let key = ChainPrivateKey::from("value");

    let res = instance
        .exec_transaction(params, fee_setting, key)
        .await
        .unwrap();
    tracing::info!("result of deploy multisig account = {:?}", res);
}

#[tokio::test]
async fn test_build_multisig_tx() {
    let from = MULTISIG_ACCOUNT_ADDR;
    let to = "address";
    let value = unit::convert_to_u256("0.01", 18).unwrap();
    let token = None;

    let params = operations::MultisigTransferOpt::new(from, to, value)
        .unwrap()
        .with_token(token)
        .unwrap();

    let instance = get_chain().build_multisig_tx(params).await.unwrap();
    tracing::info!("result of build multisig tx = {:?}", instance);
}

#[tokio::test]
async fn test_sign_transaction() {
    let input_data = "xxx".to_string();
    let sign_message = "xx".to_string();

    let params = operations::MultisigPayloadOpt::new(input_data, sign_message);

    let key = ChainPrivateKey::from("key");

    let res = params.sign_message(key).unwrap();
    tracing::info!("result of sign multisig tx = {:?}", res);
}

#[tokio::test]
async fn test_exec_multi_transaction() {
    let from = MULTISIG_ACCOUNT_ADDR;
    let to = "address";
    let value = unit::convert_to_u256("0.01", 18).unwrap();
    let token = None;

    let input_data = "".to_string();
    let sign_seq = "".to_string();
    let params = operations::MultisigTransferOpt::new(from, to, value)
        .unwrap()
        .with_token(token)
        .unwrap()
        .exec_params(from, input_data, sign_seq)
        .unwrap();

    let key = ChainPrivateKey::from("key");
    let fee_setting = FeeSetting::default();
    let res = get_chain()
        .exec_transaction(params, fee_setting, key)
        .await
        .unwrap();
    tracing::info!("result of exec multisig tx = {:?}", res);
}
