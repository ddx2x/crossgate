mod error;

use crate::object::Object;
use crate::store::{Condition, Context, Event, Filter, Stores, Stroage};

use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
pub struct Service<T: Object, F: Filter, S: Stroage<T, F>> {
    schema: String,
    table: String,
    store: Stores<T, F, S>,
    _f: Option<F>,
}

impl<T, F, S> Service<T, F, S>
where
    T: Object,
    F: Filter,
    S: Stroage<T, F>,
{
    pub fn new(schema: String, table: String, store: Stores<T, F, S>) -> Self {
        Self {
            store: store,
            schema: schema,
            table: table,
            _f: None,
        }
    }

    fn intercept(&self, q: Condition<F>) -> Condition<F> {
        let mut q = q;
        q.with_db(&self.schema).with_table(&self.table);
        q
    }

    pub async fn list(&self, q: Condition<F>) -> crate::Result<Vec<T>> {
        Ok(self.store.list(self.intercept(q)).await?)
    }

    pub async fn get(&self, q: Condition<F>) -> crate::Result<T> {
        Ok(self.store.get(self.intercept(q)).await?)
    }

    pub async fn save(&self, t: T, q: Condition<F>) -> crate::Result<()> {
        Ok(self.store.save(t, self.intercept(q)).await?)
    }

    pub async fn remove(&self, q: Condition<F>) -> crate::Result<()> {
        Ok(self.store.remove(self.intercept(q)).await?)
    }

    pub async fn watch(&self, ctx: Context, q: Condition<F>) -> Receiver<Event<T>> {
        self.store
            .watch(ctx, self.schema.to_string(), self.table.to_string(), q)
            .await
            .unwrap()
    }
}
