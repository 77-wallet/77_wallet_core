#[derive(Debug, thiserror::Error)]
pub enum GlobalValueError {
    #[error("value is not initialized")]
    ValueNotInit,
}
