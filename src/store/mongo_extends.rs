use super::{Condition, Filter};
use futures::Future;
use serde::de::DeserializeOwned;

pub trait MongoDbModel: Sync + Send + Unpin + serde::Serialize + DeserializeOwned {}

pub trait MongoStorageExtends<F: Filter>: Sync + Send + Clone + 'static {
    type ListFuture<'a, T>: Future<Output = crate::Result<Vec<T>>>
    where
        Self: 'a,
        T: MongoDbModel;

    type GetFuture<'a, T>: Future<Output = crate::Result<T>>
    where
        Self: 'a,
        T: MongoDbModel;

    type SaveFuture<'a, T>: Future<Output = crate::Result<()>>
    where
        Self: 'a,
        T: MongoDbModel;

    type UpdateFuture<'a, T>: Future<Output = crate::Result<T>>
    where
        Self: 'a,
        T: MongoDbModel;

    type RemoveFuture<'a, T>: Future<Output = crate::Result<()>>
    where
        Self: 'a,
        T: MongoDbModel;

    fn list_any_type<'r, T>(&'r self, q: Condition<F>) -> Self::ListFuture<'r, T>
    where
        T: MongoDbModel;

    fn save_any_type<'r, T>(&'r self, t: T, q: Condition<F>) -> Self::SaveFuture<'r, T>
    where
        T: MongoDbModel;

    fn update_any_type<'r, T>(&'r self, t: T, q: Condition<F>) -> Self::UpdateFuture<'r, T>
    where
        T: MongoDbModel;

    fn delete_any_type<'r, T>(&'r self, q: Condition<F>) -> Self::RemoveFuture<'r, T>
    where
        T: MongoDbModel;

    fn get_any_type<'r, T>(&'r self, q: Condition<F>) -> Self::GetFuture<'r, T>
    where
        T: MongoDbModel;
}
