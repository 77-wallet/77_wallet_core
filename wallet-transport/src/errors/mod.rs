use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("node response  {0}")]
    NodeResponseError(String),
    #[error("query result empty")]
    EmptyResult,
    #[error("Utils error: {0}")]
    Utils(#[from] wallet_utils::error::Error),
    #[error("Rumqttc v5 option error: {0}")]
    RumqttcV5Option(#[from] rumqttc::v5::OptionError),
    #[error("Aliyun oss error: {0}")]
    AliyunOss(#[from] crate::client::oss_client::error::OssError),
}

impl TransportError {
    pub fn is_network_error(&self) -> bool {
        match self {
            TransportError::Utils(e) => e.is_network_error(),
            _ => false,
        }
    }
}
