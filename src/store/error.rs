#[derive(Debug)]
pub enum StoreError {
    DataNotFound,
    DuplicateKey(String),
    ConnectionError(String),
    /// All other errors
    Other(crate::Error),
}
