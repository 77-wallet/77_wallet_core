#[derive(Debug, thiserror::Error)]
pub enum SerdeError {
    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Parse value to vector failed")]
    ValueToVecFailed,
    #[error(" deserialize error: {0}")]
    Deserialize(String),
    #[error("Toml serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("Toml deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),
    #[error("Yaml serialize error: {0}")]
    SerdeYamlError(#[from] serde_yaml::Error),
}

impl SerdeError {
    pub fn get_status_code(&self) -> u32 {
        match self {
            SerdeError::Json(_) => 6061,
            // SerdeError::BsonSer(_) => 6061,
            // SerdeError::BsonDeser(_) => 6061,
            SerdeError::ValueToVecFailed => 6062,
            SerdeError::Deserialize(_) => 6063,
            SerdeError::TomlSerialize(_) => 6064,
            SerdeError::TomlDeserialize(_) => 6065,
            SerdeError::SerdeYamlError(_) => 6066,
        }
    }
}
