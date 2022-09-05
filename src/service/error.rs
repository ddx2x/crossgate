#[derive(Debug)]
pub enum ServiceError {
    InternalError(crate::store::StoreError),
    /// All other errors
    Other(crate::Error),
}
