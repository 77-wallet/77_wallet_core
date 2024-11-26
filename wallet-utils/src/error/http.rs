#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("Get extension failed")]
    GetExtensionFailed,
    #[error("Response build failed")]
    ResponseBuildFailed,
    #[error("Invalid header")]
    InvalidHeader,
    #[error("request error {0}")]
    ReqError(#[from] reqwest::Error),
    #[error("Received a non-success status: {0}")]
    NonSuccessStatus(reqwest::StatusCode),
    // #[error("Axum error: {0}")]
    // Axum(#[from] axum::http::Error),
    // #[error("Hyper error: {0}")]
    // Hyper(#[from] hyper::Error),
}

impl HttpError {
    pub fn get_status_code(&self) -> u32 {
        match self {
            HttpError::GetExtensionFailed => 6210,
            HttpError::ResponseBuildFailed => 6211,
            HttpError::InvalidHeader => 6212,
            HttpError::ReqError(_) => 6212,
            HttpError::NonSuccessStatus(_) => 6212,
            // HttpError::Axum(_) => 6215,
            // HttpError::Hyper(_) => 6216,
        }
    }
}
