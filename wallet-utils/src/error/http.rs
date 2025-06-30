#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    GetExtensionFailed,
    ResponseBuildFailed,
    InvalidHeader,
    ReqError(#[from] reqwest::Error),
    NonSuccessStatus(reqwest::StatusCode),
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

use std::error::Error;
impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GetExtensionFailed => write!(f, "Get extension failed"),
            Self::ResponseBuildFailed => write!(f, "Response build failed"),
            Self::InvalidHeader => write!(f, "Invalid header"),
            Self::NonSuccessStatus(code) => {
                write!(f, "Received non-success HTTP status: {}", code)
            }
            Self::ReqError(e) => {
                writeln!(f, "request error: {}", e)?;
                let mut source = e.source();
                while let Some(s) = source {
                    writeln!(f, "caused by: {}", s)?;
                    source = s.source();
                }
                write!(
                    f,
                    "is_timeout: {}, is_connect: {}, url: {:?}",
                    e.is_timeout(),
                    e.is_connect(),
                    e.url()
                )
            }
        }
    }
}
