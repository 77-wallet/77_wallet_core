use std::fmt::Debug;

#[derive(Debug, serde::Serialize)]
pub struct JsonRpcParams<T: Debug + serde::Serialize> {
    id: u64,
    jsonrpc: String,
    method: String,
    params: Option<T>,
}

impl<T: Debug + serde::Serialize> Default for JsonRpcParams<T> {
    fn default() -> Self {
        Self {
            id: 1,
            jsonrpc: "2.0".to_string(),
            method: "".to_string(),
            params: None,
        }
    }
}

impl<T: Debug + serde::Serialize> JsonRpcParams<T> {
    pub fn method(mut self, method: &str) -> Self {
        self.method = method.to_string();
        self
    }
    pub fn params(mut self, params: T) -> Self {
        self.params = Some(params);
        self
    }

    pub fn no_params(mut self) -> Self {
        self.params = None;
        self
    }
}
