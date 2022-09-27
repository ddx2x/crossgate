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

    type StreamFuture<'a>: Future<Output = Receiver<oplog::Event<T>>>
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
    ) -> impl Future<Output = Receiver<oplog::Event<T>>> + 'r {
        async move { self.store.watch(ctx, db, table, q).await }
    }
}

#[cfg(test)]
mod test {
    // use super::{cond::Condition, Query, Value, DB, TABLE};
    #[test]
    fn test_condition() {
        // let query: Query<&str, Value> =
        //     query!(DB => Value::String("pub"),TABLE => Value::String("user"));
        // let condition = Condition::parse(query);
        // assert_eq!(condition.db, "pub");
        // assert_eq!(condition.table, "user");
    }
}
