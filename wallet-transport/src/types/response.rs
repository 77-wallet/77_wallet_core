use std::fmt::Debug;
#[derive(Debug, serde::Deserialize)]
pub struct JsonRpcResult<T, E = String> {
    pub result: T,
    pub error: Option<E>,
}

#[derive(Debug, serde::Deserialize)]
pub struct RpcResult<T = String, E = JsonRpcError> {
    pub result: Option<T>,
    pub error: Option<E>,
}

#[derive(Debug, serde::Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<String>,
}
