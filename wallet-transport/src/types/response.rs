use std::fmt::Debug;

#[derive(Debug, serde::Deserialize)]
pub struct JsonRpcResult<T = String, E = JsonRpcError> {
    pub result: Option<T>,
    pub error: Option<E>,
}

// rpc 请求的error 信息
#[derive(Debug, serde::Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
