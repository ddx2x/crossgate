mod error;

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
        Ok(self.storage.list(self.intercept(q)).await?)
    }

    pub async fn get(&self, q: Condition<F>) -> crate::Result<T> {
        Ok(self.storage.get(self.intercept(q)).await?)
    }

    pub async fn save(&self, t: T, q: Condition<F>) -> crate::Result<()> {
        Ok(self.storage.save(t, self.intercept(q)).await?)
    }

    pub async fn remove(&self, q: Condition<F>) -> crate::Result<()> {
        Ok(self.storage.delete(self.intercept(q)).await?)
    }

    pub async fn watch(&self, ctx: Context, q: Condition<F>) -> crate::Result<Receiver<Event<T>>> {
        self.storage
            .watch(ctx, self.schema.to_string(), self.table.to_string(), q)
            .await
    }
}
