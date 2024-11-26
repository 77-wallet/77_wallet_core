use crate::tron::get_chain;
use wallet_chain_interact::tron::operations::{self, TronConstantOperation, TronTxOperation};
use wallet_chain_interact::types::ChainPrivateKey;
use wallet_utils::unit;

#[tokio::test]
async fn test_balance() {
    let instance = get_chain();
    let address = "address";
    let token: Option<String> = None;

    let res = instance.balance(address, token).await;
    tracing::info!("balance of = {:?}", res);
    assert!(res.is_ok())
}

#[tokio::test]
async fn test_estimate_fee() {
    let from = "address";
    let to = "address";
    let value = unit::convert_to_u256("53.075021", 6).unwrap();
    let memo = None;
    // let memo = Some("test".to_string());

    let params = operations::transfer::TransferOpt::new(from, to, value, memo).unwrap();
    let res = get_chain()
        .simulate_simple_fee(from, to, 1, params)
        .await
        .unwrap();

    tracing::info!("brand consumer = {:?}", res);
    tracing::info!("transaction fee = {:?}", res.transaction_fee());
}

#[tokio::test]
async fn test_estimate_token_fee() {
    let contract = "contract address";

    let from = "address";
    let to = "address";
    let value = unit::convert_to_u256("1", 6).unwrap();
    let memo = Some("test".to_string());

    let params =
        operations::transfer::ContractTransferOpt::new(&contract, from, to, value, memo).unwrap();

    let res = get_chain().contract_fee(&from, 1, params).await.unwrap();

    tracing::info!("brand consumer = {:#?}", res);
    tracing::info!("transaction fee = {:?}", res.transaction_fee());
}

#[tokio::test]
async fn test_transfer() {
    let from = "address";
    let to = "address";
    let value = unit::convert_to_u256("1", 6).unwrap();

    let params = operations::transfer::TransferOpt::new(from, to, value, None).unwrap();

    let key = "private key";
    let key = ChainPrivateKey::from(key);

    let chain = get_chain();
    let raw = params
        .build_raw_transaction(chain.get_provider())
        .await
        .unwrap();
    let instance = chain.exec_transaction_v1(raw, key).await.unwrap();

    tracing::info!("tx info of = {:?}", instance);
}

#[tokio::test]
async fn test_token_transfer_fee() {
    let contract = "contract address";

    let from = "address";
    let to = "address";
    let value = unit::convert_to_u256("0.1", 6).unwrap();
    let memo = Some("test".to_string());

    let params =
        operations::transfer::ContractTransferOpt::new(&contract, &from, &to, value, memo).unwrap();

    let res = get_chain().contract_fee(&from, 1, params).await.unwrap();

    tracing::info!("brand consumer = {:#?}", res);
    tracing::info!("transaction fee = {:?}", res.transaction_fee());
}

#[tokio::test]
async fn test_token_transfer() {
    let contract = "contract address";

    let from = "address";
    let to = "address";
    let value = unit::convert_to_u256("0.1", 6).unwrap();
    let memo = Some("test".to_string());

    let mut params =
        operations::transfer::ContractTransferOpt::new(&contract, &from, &to, value, memo).unwrap();

    let key = "private key";
    let key = ChainPrivateKey::from(key);

    let chain = get_chain();
    let c = params
        .constant_contract(chain.get_provider())
        .await
        .unwrap();

    let fee = chain.provider.contract_fee(c, 1, &from).await.unwrap();
    params.set_fee_limit(fee);

    let raw = params
        .build_raw_transaction(chain.get_provider())
        .await
        .unwrap();
    let tx = chain.exec_transaction_v1(raw, key).await.unwrap();

    tracing::info!("tx info of = {:?}", tx);
}

#[tokio::test]
async fn test_decimals() {
    let instance = get_chain();

    let token = "contract address";

    let res = instance.decimals(token).await.unwrap();
    tracing::info!("decimals = {:?}", res);

    let name = instance.token_name(token).await.unwrap();
    tracing::info!("name = {:?}", name);

    let symbol = instance.token_symbol(token).await.unwrap();
    tracing::info!("symbol = {:?}", symbol);
}

#[tokio::test]
async fn test_query_tx() {
    let instance = get_chain();
    let tx = "53551f65e23d49b85a9f665c80a450dfe07325a9e131a5c4e88d0ce55dfd9770";
    let res = instance.query_tx_res(tx).await;
    tracing::info!("tx info of = {:?}", res);
    assert!(res.is_ok())
}

#[tokio::test]
async fn test_account_resource() {
    let instance = get_chain();
    let res = instance.provider.account_resource("address").await.unwrap();
    tracing::info!("tx info of = {:#?}", res);
}

#[tokio::test]
pub async fn test_parameter() {
    let chain = get_chain();
    let parameter = chain.provider.chain_params().await.unwrap();

    let fee = parameter.get_multi_sign_fee();
    tracing::info!("create account fee {:?}", fee)
}

#[tokio::test]
pub async fn test_block_height() {
    let chain = get_chain();
    let block = chain.provider.get_block().await.unwrap();
    tracing::info!("block height {:?}", block.block_header.raw_data.number)
}

#[tokio::test]
pub async fn test_chain_params() {
    let chain = get_chain();
    let parameter = chain.provider.chain_params().await.unwrap();
    tracing::info!("block height {:#?}", parameter)
}

#[tokio::test]
pub async fn test_all() {
    let chain = get_chain();

    let from = "address";
    let to = "address";
    let value = unit::convert_to_u256("53.075021", 6).unwrap();

    let params =
        operations::transfer::TransferOpt::new(from, to, value, Some("sss".to_string())).unwrap();

    let provider = chain.get_provider();
    let tx = params.build_raw_transaction(provider).await.unwrap();
    let consumer = provider
        .transfer_fee(&params.from, Some(&params.to), &tx, 1)
        .await
        .unwrap();

    let account = provider.account_info(&params.from).await.unwrap();

    tracing::warn!("consumer {:#?}", consumer);
    tracing::warn!("consumer {:#?}", account);
}

#[tokio::test]
async fn test_black_address() {
    let chain = get_chain();

    let token = "token address";
    let owner = "address";

    let res = chain.black_address(token, owner).await.unwrap();
    tracing::warn!("is black address {}", res);
}
