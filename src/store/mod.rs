mod mongo;
pub use mongo::*;
mod mongo_extends;
pub use mongo_extends::{MongoDbModel, MongoStorageExtends};
mod condition;
pub use self::condition::Condition;
mod error;
pub use error::StoreError;

use futures::Future;
use std::{
    fmt::Debug,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc::Receiver;

use crate::object::Object;

pub type Context = tokio_context::context::Context;

pub fn current_time_sess() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Event<T> {
    Added(T),
    Updated(T),
    Deleted(T),
    Error(String),
}

impl<T> std::fmt::Display for Event<T>
where
    T: std::fmt::Debug + serde::Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Added(ref t) => write!(
                f,
                r#"{{ \"type\": \"ADDED\", \"object\": {:?} }}"#,
                serde_json::to_string(&t).unwrap()
            ),
            Event::Updated(t) => write!(
                f,
                r#"{{ \"type\": \"MODIFIED\", \"object\": {:?} }}"#,
                serde_json::to_string(&t).unwrap()
            ),
            Event::Deleted(t) => write!(
                f,
                r#"{{ \"type\": \"DELETED\", \"object\": {:?} }}"#,
                serde_json::to_string(&t).unwrap()
            ),
            Event::Error(ref s) => write!(f, r#"{{ \"type\": \"ERROR\", \"msg\": {:?} }}"#, s),
        }
    }
}

pub trait Filter: Clone + Debug {
    fn parse(&mut self, input: &str) -> anyhow::Result<Box<Self>>;
}

pub trait Storage<T: Object, F: Filter>: Sync + Send + Clone + 'static {
    type ListFuture<'a>: Future<Output = crate::Result<Vec<T>>>
    where
        Self: 'a;

    type GetFuture<'a>: Future<Output = crate::Result<T>>
    where
        Self: 'a;

    type SaveFuture<'a>: Future<Output = crate::Result<()>>
    where
        Self: 'a;

    type UpdateFuture<'a>: Future<Output = crate::Result<T>>
    where
        Self: 'a;

    type RemoveFuture<'a>: Future<Output = crate::Result<()>>
    where
        Self: 'a;

    type StreamFuture<'a>: Future<Output = crate::Result<Receiver<Event<T>>>>
    where
        Self: 'a;

    fn save<'r>(&'r self, t: T, q: Condition<F>) -> Self::SaveFuture<'r>;

    fn update<'r>(&'r self, t: T, q: Condition<F>) -> Self::UpdateFuture<'r>;

    fn delete<'r>(&'r self, q: Condition<F>) -> Self::RemoveFuture<'r>;

    fn list<'r>(&'r self, q: Condition<F>) -> Self::ListFuture<'r>;

    fn get<'r>(&'r self, q: Condition<F>) -> Self::GetFuture<'r>;

    fn watch<'r>(
        &'r self,
        ctx: Context,
        db: String,
        table: String,
        q: Condition<F>,
    ) -> Self::StreamFuture<'r>;
}
