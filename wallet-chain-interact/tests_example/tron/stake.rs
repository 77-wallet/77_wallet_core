use crate::tron::get_chain_stake;
use wallet_chain_interact::tron::params;
use wallet_chain_interact::types::ChainPrivateKey;

#[tokio::test]
async fn test_freeze_balance() {
    let chain = get_chain_stake();
    let key = ChainPrivateKey::from("private key");

    let owner_address = "address";
    let resource = "energy";
    let freeze_balance = "100";

    let args = params::FreezeBalanceArgs::new(owner_address, &resource, freeze_balance).unwrap();
    let resp = chain.freeze_balance(args, key).await;

    tracing::info!("resp: {:?}", resp);
}

#[tokio::test]
async fn test_un_freeze_balance() {
    let chain = get_chain_stake();
    let key = ChainPrivateKey::from("private key");

    let owner_address = "address";
    let resource = "energy";
    let freeze_balance = "100";

    let args = params::UnFreezeBalanceArgs::new(owner_address, &resource, freeze_balance).unwrap();
    let resp = chain.unfreeze_balance(args, key).await;

    tracing::info!("resp: {:?}", resp);
}

#[tokio::test]
async fn test_delegate_resource() {
    let chain = get_chain_stake();
    let key = ChainPrivateKey::from("private key");

    let owner_address = "address";
    let receiver_address = "address";
    let balance = "50";
    let resource = "energy";

    let args =
        params::DelegateArgs::new(owner_address, receiver_address, balance, &resource).unwrap();
    let resp = chain.delegate_resource(args, key).await;
    tracing::info!("resp: {:?}", resp);
}

#[tokio::test]
async fn test_un_delegate_resource() {
    let chain = get_chain_stake();
    let key = ChainPrivateKey::from("private key");

    let owner_address = "address";
    let receiver_address = "adress";
    let balance = "100";
    let resource = "energy";

    let args =
        params::UnDelegateArgs::new(owner_address, receiver_address, balance, &resource).unwrap();
    let resp = chain.un_delegate_resource(args, key).await;
    tracing::info!("resp: {:?}", resp);
}

#[tokio::test]
async fn test_can_withdraw_freeze_amount() {
    let chain = get_chain_stake();

    let owner_address = "address";
    let resp = chain.can_withdraw_unfreeze_amount(&owner_address).await;
    tracing::info!("resp: {:?}", resp);
}

#[tokio::test]
async fn test_account1() {
    let chain = get_chain_stake();
    let owner_address = "address";
    let resp = chain.account_info(&owner_address).await;
    tracing::info!("resp: {:?}", resp);
}
