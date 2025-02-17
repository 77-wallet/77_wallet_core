use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("node response: {}", .0)]
    NodeResponseError(NodeResponseError),
    #[error("query result empty")]
    EmptyResult,
    #[error("Utils error: {0}")]
    Utils(#[from] wallet_utils::error::Error),
    #[error("Rumqttc v5 option error: {0}")]
    RumqttcV5Option(#[from] rumqttc::v5::OptionError),
}

impl std::fmt::Display for NodeResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node response error: code={}, message={:?}",
            self.code, self.message
        )
    }
}
impl TransportError {
    pub fn is_network_error(&self) -> bool {
        match self {
            TransportError::Utils(e) => e.is_network_error(),
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct NodeResponseError {
    pub code: i64,
    pub message: Option<String>,
}

impl NodeResponseError {
    pub fn new(code: i64, message: Option<String>) -> Self {
        Self { code, message }
    }
}
