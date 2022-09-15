use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error(r#"data store error `{0}`"#)]
    InternalError(crate::store::StoreError),
    #[error("unknown service error")]
    Other(crate::Error),
}
