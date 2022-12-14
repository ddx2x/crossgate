//export MongoStoreService
mod mongo_service;
pub use mongo_service::MongoStoreService;

use crate::object::Object;
use crate::store::{Condition, Context, Event, Filter, Storage};

use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
pub struct Service<T: Object, F: Filter, S: Storage<T, F>> {
    schema: String,
    table: String,
    storage: S,
    _ref: Option<(T, F)>,
}

impl<T, F, S> Service<T, F, S>
where
    T: Object,
    F: Filter,
    S: Storage<T, F>,
{
    pub fn new(schema: String, table: String, storage: S) -> Self {
        Self {
            storage,
            schema,
            table,
            _ref: None,
        }
    }

    fn intercept(&self, q: Condition<F>) -> Condition<F> {
        let mut q = q;
        q.with_db(&self.schema).with_table(&self.table);
        q
    }

    pub async fn list(&self, q: Condition<F>) -> crate::Result<Vec<T>> {
        self.storage.clone().list(self.intercept(q)).await
    }

    pub async fn get(&self, q: Condition<F>) -> crate::Result<T> {
        self.storage.clone().get(self.intercept(q)).await
    }

    pub async fn apply(&self, t: T, q: Condition<F>) -> crate::Result<T> {
        self.storage.clone().apply(t, self.intercept(q)).await
    }

    pub async fn update(&self, t: T, q: Condition<F>) -> crate::Result<()> {
        self.storage.clone().update(t, self.intercept(q)).await
    }

    pub async fn save(&self, t: T, q: Condition<F>) -> crate::Result<()> {
        self.storage.clone().save(t, self.intercept(q)).await
    }

    pub async fn remove(&self, q: Condition<F>) -> crate::Result<()> {
        self.storage.clone().delete(self.intercept(q)).await
    }

    pub async fn count(&self, q: Condition<F>) -> crate::Result<u64> {
        self.storage.clone().count(self.intercept(q)).await
    }

    pub async fn watch(&self, ctx: Context, q: Condition<F>) -> crate::Result<Receiver<Event<T>>> {
        self.storage.clone().watch(ctx, self.intercept(q)).await
    }
}
