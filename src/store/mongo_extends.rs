use super::{Condition, Filter};
use futures::Future;
use serde::de::DeserializeOwned;

pub trait MongoDbModel: Sync + Send + Unpin + serde::Serialize + DeserializeOwned {}

pub trait MongoStorageExtends<F: Filter>: Sync + Send + Clone + 'static {
    type ListFuture<'a, T>: Future<Output = crate::Result<Vec<T>>>
    where
        Self: 'a,
        T: MongoDbModel;

    fn list_any_type<'r, T>(&'r self, q: Condition<F>) -> Self::ListFuture<'r, T>
    where
        T: MongoDbModel;
}
