use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum StoreError {
    #[error("data not found")]
    DataNotFound,
    #[error(r#"the data for key `{0}` is duplicate"#)]
    DuplicateKey(String),
    #[error("store connection abnormal info `{0}`")]
    ConnectionError(String),
}
