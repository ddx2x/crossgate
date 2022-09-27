mod error;

use crate::object::Object;
use crate::store::{Condition, Context, Filter, Stores, Stroage};

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

    fn query(&self, q: Condition<F>) -> Condition<F> {
        let mut q = q;
        q.with_db(&self.schema).with_table(&self.table);
        q
    }

    pub async fn list(&self, q: Condition<F>) -> crate::Result<Vec<T>> {
        Ok(self.store.list(self.query(q)).await?)
    }

    pub async fn get(&self, q: Condition<F>) -> crate::Result<T> {
        Ok(self.store.get(self.query(q)).await?)
    }

    pub async fn save(&self, t: T, q: Condition<F>) -> crate::Result<()> {
        Ok(self.store.save(t, self.query(q)).await?)
    }

    pub async fn remove(&self, q: Condition<F>) -> crate::Result<()> {
        Ok(self.store.remove(self.query(q)).await?)
    }

    pub async fn watch(&self, ctx: Context, q: Condition<F>) -> Receiver<oplog::Event<T>> {
        self.store
            .watch(ctx, self.schema.to_string(), self.table.to_string(), q)
            .await
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::{
//         object::{decorate, Object},
//         query,
//         store::MongoStore,
//     };

//     #[decorate]
//     struct Local {}

//     #[tokio::test]
//     async fn test_service() {
//         let m = MongoStore::new("mongodb://10.200.100.200:27017")
//             .await
//             .unwrap();
//         let s = Service::<Local, MongoStore>::new(
//             "base".to_string(),
//             "local".to_string(),
//             Stores::new(m),
//         );

//         let q = query!();
//         if let Ok(v) = s.list(q.clone()).await {
//             println!("{:?}", v);
//         } else {
//             panic!("test error")
//         }

//         // if let Ok(v) = s.get(q.clone()).await {
//         //     println!("{:?}", v);
//         //     return;
//         // } else {
//         //     panic!("test error")
//         // }
//     }
// }
