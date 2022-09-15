use futures::Future;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

mod mongo;
pub use mongo::MongoStore;

mod error;
pub use error::StoreError;

use crate::object::Object;

mod condition;

pub const DB: &'static str = "db";
pub const TABLE: &'static str = "table";
pub const UID: &'static str = "uid";

pub type Context = tokio_context::context::Context;

// "UID": {Equal: "123"}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PredicateOperator {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Like,
    NotLike,
    In,
    NotIn,
    Between,
    NotBetween,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone)]
pub enum Value<'a> {
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value<'a>>),
    Pair(HashMap<PredicateOperator, Value<'a>>),
    Null,
}

pub type Query<K, V> = HashMap<K, V>;

#[macro_export]
macro_rules! query {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(query!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { query!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = query!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

pub trait Stroage<T: Object>: Sync + Send + Clone + 'static {
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

    fn save<'r>(&'r self, t: T, q: Query<&str, Value<'r>>) -> Self::SaveFuture<'r>;

    fn delete<'r>(&'r self, q: Query<&str, Value<'r>>) -> Self::RemoveFuture<'r>;

    fn list<'r>(&'r self, q: Query<&'r str, Value<'r>>) -> Self::ListFuture<'r>;

    fn get<'r>(&'r self, q: Query<&'r str, Value<'r>>) -> Self::GetFuture<'r>;

    fn watch<'r>(
        &'r self,
        ctx: Context,
        db: String,
        table: String,
        q: Query<&str, Value<'_>>,
    ) -> Self::StreamFuture<'r>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Stores<T: Object, S: Stroage<T>> {
    _t: Option<T>,
    store: S,
}

impl<T, S> Stores<T, S>
where
    T: Object,
    S: Stroage<T>,
{
    pub fn new(s: S) -> Self {
        Stores { _t: None, store: s }
    }

    pub fn get<'r>(
        &'r self,
        q: Query<&'r str, Value<'r>>,
    ) -> impl Future<Output = crate::Result<T>> + 'r {
        async move { self.store.get(q).await }
    }

    pub fn save<'r>(
        &'r self,
        t: T,
        q: Query<&'r str, Value<'r>>,
    ) -> impl Future<Output = crate::Result<()>> + 'r {
        async move { self.store.save(t, q).await }
    }

    pub fn remove<'r>(
        &'r self,
        q: Query<&'r str, Value<'r>>,
    ) -> impl Future<Output = crate::Result<()>> + 'r {
        async move { self.store.delete(q).await }
    }

    pub fn list<'r>(
        &'r self,
        q: Query<&'r str, Value<'r>>,
    ) -> impl Future<Output = crate::Result<Vec<T>>> + 'r {
        async move { self.store.list(q).await }
    }

    pub fn watch<'r>(
        &'r self,
        ctx: Context,
        db: String,
        table: String,
        q: Query<&'r str, Value<'r>>,
    ) -> impl Future<Output = Receiver<oplog::Event<T>>> + 'r {
        async move { self.store.watch(ctx, db, table, q).await }
    }
}

#[cfg(test)]
mod test {
    use super::{condition::Condition, Query, Value, DB, TABLE};
    #[test]
    fn test_condition() {
        let query: Query<&str, Value> =
            query!(DB => Value::String("pub"),TABLE => Value::String("user"));
        let condition = Condition::parse(query);
        assert_eq!(condition.db, "pub");
        assert_eq!(condition.table, "user");
    }
}
