mod mongo;
pub use mongo::*;
mod condition;
pub use self::condition::Condition;
mod error;
pub use error::StoreError;

use futures::Future;
use std::fmt::Debug;
use tokio::sync::mpsc::Receiver;

use crate::object::Object;

pub type Context = tokio_context::context::Context;

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

pub trait Stroage<T: Object, F: Filter>: Sync + Send + Clone + 'static {
    type ListFuture<'a>: Future<Output = crate::Result<Vec<T>>>
    where
        Self: 'a;

    type GetFuture<'a>: Future<Output = crate::Result<T>>
    where
        Self: 'a;

    type SaveFuture<'a>: Future<Output = crate::Result<()>>
    where
        Self: 'a;

    type RemoveFuture<'a>: Future<Output = crate::Result<()>>
    where
        Self: 'a;

    type StreamFuture<'a>: Future<Output = crate::Result<Receiver<Event<T>>>>
    where
        Self: 'a;

    fn save<'r>(&'r self, t: T, q: Condition<F>) -> Self::SaveFuture<'r>;

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Stores<T: Object, F: Filter, S: Stroage<T, F>> {
    _t: Option<T>,
    _f: Option<F>,
    store: S,
}

impl<T, F, S> Stores<T, F, S>
where
    T: Object,
    F: Filter,
    S: Stroage<T, F>,
{
    pub fn new(s: S) -> Self {
        Stores {
            _t: None,
            _f: None,
            store: s,
        }
    }

    pub fn get<'r>(&'r self, q: Condition<F>) -> impl Future<Output = crate::Result<T>> + 'r {
        async move { self.store.get(q).await }
    }

    pub fn save<'r>(
        &'r self,
        t: T,
        q: Condition<F>,
    ) -> impl Future<Output = crate::Result<()>> + 'r {
        async move { self.store.save(t, q).await }
    }

    pub fn remove<'r>(&'r self, q: Condition<F>) -> impl Future<Output = crate::Result<()>> + 'r {
        async move { self.store.delete(q).await }
    }

    pub fn list<'r>(&'r self, q: Condition<F>) -> impl Future<Output = crate::Result<Vec<T>>> + 'r {
        async move { self.store.list(q).await }
    }

    pub fn watch<'r>(
        &'r self,
        ctx: Context,
        db: String,
        table: String,
        q: Condition<F>,
    ) -> impl Future<Output = crate::Result<Receiver<Event<T>>>> + 'r {
        async move { self.store.watch(ctx, db, table, q).await }
    }
}
