pub trait Transaction<T> {
    fn build_transaction(&self) -> Result<T, crate::Error>;
}
