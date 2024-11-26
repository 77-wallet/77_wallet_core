use serde::Deserialize;
pub mod account;
pub mod block;
pub mod contract;
pub mod transaction;

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub context: Context,
    pub value: T,
}

#[derive(Debug, Deserialize)]
pub struct Context {
    pub slot: u128,
}
