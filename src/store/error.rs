use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("data not found")]
    DataNotFound,
    #[error(r#"the data for key `{0}` is duplicate"#)]
    DuplicateKey(String),
    #[error("store connection abnormal info `{0}`")]
    ConnectionError(String),
    #[error("unknown data store error: `{0}`")]
    Other(crate::Error),
}
