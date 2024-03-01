use crate::utils;

use super::{Condition, Context, Event, Filter};
use crate::store::Result;
use bson::Document;
use futures::Future;
use serde::de::DeserializeOwned;
use tokio::sync::mpsc::Receiver;

pub trait MongoDbModel: Sync + Send + Unpin + serde::Serialize + DeserializeOwned {}

impl MongoDbModel for utils::Unstructed {}

pub trait MongoStorageExtends<F: Filter>: Sync + Send + Clone + 'static {
    type ListFuture<'a, T>: Future<Output = Result<Option<Vec<T>>>>
    where
        Self: 'a,
        T: MongoDbModel;

    fn list_any_type<'r, T>(self, q: Condition<F>) -> Self::ListFuture<'r, T>
    where
        T: MongoDbModel;

    type SaveFuture<'a, T>: Future<Output = Result<Option<T>>>
    where
        Self: 'a,
        T: MongoDbModel;
    fn save_any_type<'r, T>(self, t: T, q: Condition<F>) -> Self::SaveFuture<'r, T>
    where
        T: MongoDbModel;

    type ApplyFuture<'a, T>: Future<Output = Result<Option<T>>>
    where
        Self: 'a,
        T: MongoDbModel;
    fn apply_any_type<'r, T>(self, t: T, q: Condition<F>) -> Self::ApplyFuture<'r, T>
    where
        T: MongoDbModel;

    type UpdateFuture<'a, T>: Future<Output = Result<Option<T>>>
    where
        Self: 'a,
        T: MongoDbModel;
    fn update_any_type<'r, T>(self, t: T, q: Condition<F>) -> Self::UpdateFuture<'r, T>
    where
        T: MongoDbModel;

    type RemoveFuture<'a, T>: Future<Output = Result<()>>
    where
        Self: 'a,
        T: MongoDbModel;
    fn delete_any_type<'r, T>(self, q: Condition<F>) -> Self::RemoveFuture<'r, T>
    where
        T: MongoDbModel;

    type GetFuture<'a, T>: Future<Output = Result<Option<T>>>
    where
        Self: 'a,
        T: MongoDbModel;
    fn get_any_type<'r, T>(self, q: Condition<F>) -> Self::GetFuture<'r, T>
    where
        T: MongoDbModel;

    type StreamFuture<'a, T>: Future<Output = Result<Receiver<Event<T>>>>
    where
        Self: 'a,
        T: MongoDbModel + 'static;
    fn watch_any_type<'r, T>(self, ctx: Context, q: Condition<F>) -> Self::StreamFuture<'r, T>
    where
        T: MongoDbModel + 'static;

    type CountFuture<'a, T>: Future<Output = Result<u64>>
    where
        Self: 'a,
        T: MongoDbModel;
    fn count<'r, T>(self, q: Condition<F>) -> Self::CountFuture<'r, T>
    where
        T: MongoDbModel;

    type UpdateManyFuture<'a, T>: Future<Output = Result<u32>>
    where
        Self: 'a,
        T: MongoDbModel;
    fn update_many_any_type<'r, T>(self, t: T, q: Condition<F>) -> Self::UpdateManyFuture<'r, T>
    where
        T: MongoDbModel;
}

pub trait MongoStorageAggregationExtends: Sync + Send + Clone + 'static {
    type AggregationListFuture<'a, T>: Future<Output = Result<Option<Vec<T>>>>
    where
        Self: 'a,
        T: MongoDbModel;

    fn aggregate<'r, T>(
        self,
        db: String,
        table: String,
        q: Vec<Document>,
    ) -> Self::AggregationListFuture<'r, T>
    where
        T: MongoDbModel;

    type AggregationFuture<'a, T>: Future<Output = Result<Option<T>>>
    where
        Self: 'a,
        T: MongoDbModel;

    fn aggregate_one<'r, T>(
        self,
        db: String,
        table: String,
        q: Vec<Document>,
    ) -> Self::AggregationFuture<'r, T>
    where
        T: MongoDbModel;
}

pub trait MongoStorageOpExtends<F: Filter>: Sync + Send + Clone + 'static {
    type IncrFuture<'a, T>: Future<Output = Result<Option<T>>>
    where
        Self: 'a,
        T: MongoDbModel;

    fn incr<'r, T>(self, kv_pairs: &'r [(&str, i64)], q: Condition<F>) -> Self::IncrFuture<'r, T>
    where
        T: MongoDbModel;

    type BatchRemoveFuture<'a>: Future<Output = Result<u64>>
    where
        Self: 'a;

    fn batch_remove<'r>(self, q: Condition<F>) -> Self::BatchRemoveFuture<'r>;
}
