use wallet_chain_interact::tron::{Provider, TronBlockChain, TronChain, TronProvider};
use wallet_transport::client::HttpClient;
use wallet_utils::init_test_log;
mod account;
mod base;
mod multisig;
mod stake;

fn get_chain() -> TronChain {
    init_test_log();
    let url = "https://api.trongrid.io";

    let http_client = HttpClient::new(&url, None).unwrap();
    let provider = Provider::new(http_client).unwrap();

    TronChain::new(provider).unwrap()
}

fn get_chain_stake() -> TronBlockChain {
    init_test_log();
    let url = "https://api.trongrid.io";
    let provider = TronProvider::new(url).unwrap();
    TronBlockChain::new(provider).unwrap()
}
