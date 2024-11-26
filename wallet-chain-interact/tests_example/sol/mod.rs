mod address;
mod base;
mod multisig;
use std::collections::HashMap;

use wallet_chain_interact::sol::{Provider, SolanaChain};
use wallet_transport::client::RpcClient;
use wallet_utils::init_test_log;

fn get_chain() -> SolanaChain {
    init_test_log();
    let rpc = "https://api.devnet.solana.com";

    let header = None;
    let client = RpcClient::new(&rpc, header).unwrap();
    let provider = Provider::new(client).unwrap();

    SolanaChain::new(provider).unwrap()
}
