//export MongoStoreService
mod mongo_service;
pub use mongo_service::MongoStoreService;

use crate::object::Object;
use crate::store::{Condition, Context, Event, Filter, Storage, StoreError};

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
        match self.storage.clone().list(self.intercept(q)).await {
            Ok(rs) => Ok(rs),
            Err(e) => {
                if StoreError::DataNotFound.eq(&e) {
                    return Ok(vec![]);
                }
                return Err(anyhow::anyhow!("{}", e.to_string()));
            }
        }
    }

    pub async fn get(&self, q: Condition<F>) -> crate::Result<Option<T>> {
        match self.storage.clone().get(self.intercept(q)).await {
            Ok(rs) => Ok(Some(rs)),
            Err(e) => {
                if StoreError::DataNotFound.eq(&e) {
                    return Ok(None);
                }
                return Err(anyhow::anyhow!("{}", e.to_string()));
            }
        }
    }

    pub async fn apply(&self, t: T, q: Condition<F>) -> crate::Result<Option<T>> {
        match self.storage.clone().apply(t, self.intercept(q)).await {
            Ok(rs) => Ok(Some(rs)),
            Err(e) => {
                if StoreError::DataNotFound.eq(&e) {
                    return Ok(None);
                }
                return Err(anyhow::anyhow!("{}", e.to_string()));
            }
        }
    }

    pub async fn update(&self, t: T, q: Condition<F>) -> crate::Result<()> {
        match self.storage.clone().update(t, self.intercept(q)).await {
            Ok(_) => Ok(()),
            Err(e) => return Err(anyhow::anyhow!("{}", e.to_string())),
        }
    }

    pub async fn save(&self, t: T, q: Condition<F>) -> crate::Result<()> {
        match self.storage.clone().save(t, self.intercept(q)).await {
            Ok(_) => Ok(()),
            Err(e) => return Err(anyhow::anyhow!("{}", e.to_string())),
        }
    }

    pub async fn remove(&self, q: Condition<F>) -> crate::Result<()> {
        match self.storage.clone().delete(self.intercept(q)).await {
            Ok(_) => Ok(()),
            Err(e) => return Err(anyhow::anyhow!("{}", e.to_string())),
        }
    }

    pub async fn count(&self, q: Condition<F>) -> crate::Result<u64> {
        match self.storage.clone().count(self.intercept(q)).await {
            Ok(rs) => Ok(rs),
            Err(e) => return Err(anyhow::anyhow!("{}", e.to_string())),
        }
    }

    pub async fn watch(&self, ctx: Context, q: Condition<F>) -> crate::Result<Receiver<Event<T>>> {
        match self.storage.clone().watch(ctx, self.intercept(q)).await {
            Ok(t) => Ok(t),
            Err(e) => return Err(anyhow::anyhow!("{}", e.to_string())),
        }
    }
}
