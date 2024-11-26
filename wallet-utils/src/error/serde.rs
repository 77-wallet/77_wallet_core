#[derive(Debug, thiserror::Error)]
pub enum SerdeError {
    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Parse value to vector failed")]
    ValueToVecFailed,
    #[error(" deserialize error: {0}")]
    Deserialize(String),
}

impl SerdeError {
    pub fn get_status_code(&self) -> u32 {
        match self {
            SerdeError::Json(_) => 6061,
            // SerdeError::BsonSer(_) => 6061,
            // SerdeError::BsonDeser(_) => 6061,
            SerdeError::ValueToVecFailed => 6062,
            SerdeError::Deserialize(_) => 6063,
        }
    }
}
