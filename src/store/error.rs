#[derive(Debug)]
pub enum StoreError {
    DataNotFound,
    DuplicateKey(String),
    ConnectionError,
    /// All other errors
    Other(crate::Error),
}
