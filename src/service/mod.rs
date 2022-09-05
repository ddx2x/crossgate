mod error;

use crate::object::Object;
use crate::query;
use crate::store::{self, Context, Query, Stores, Stroage, Value};

use error::ServiceError;
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
pub struct Service<T: Object, S: Stroage<T>> {
    schema: String,
    table: String,
    store: Stores<T, S>,
}

impl<T, S> Service<T, S>
where
    T: Object,
    S: Stroage<T>,
{
    pub fn new(schema: String, table: String, store: Stores<T, S>) -> Self {
        Self {
            store: store,
            schema: schema,
            table: table,
        }
    }

    fn query<'a>(&'a self, q: &'a Query<&str, Value>) -> Query<&str, Value> {
        let q = q.clone();
        let mut res = query!(
            store::DB=>store::Value::String(&self.schema),
            store::TABLE=>store::Value::String(&self.table),
        );
        res.extend(q);
        res
    }

    pub async fn list(&self, q: Query<&str, Value<'_>>) -> Result<Vec<T>, ServiceError> {
        match self.store.list(self.query(&q)).await {
            Ok(r) => Ok(r),
            Err(e) => Err(ServiceError::InternalError(e)),
        }
    }

    pub async fn get(&self, q: Query<&str, Value<'_>>) -> Result<T, ServiceError> {
        match self.store.get(self.query(&q)).await {
            Ok(r) => Ok(r),
            Err(e) => Err(ServiceError::InternalError(e)),
        }
    }

    pub async fn watch(
        &self,
        ctx: Context,
        q: Query<&str, Value<'_>>
    ) -> Receiver<oplog::Event<T>> {
        self.store
            .watch(ctx, self.schema.to_string(), self.table.to_string(), q)
            .await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        object::{decorate, Object},
        query,
        store::MongoStore,
    };

    #[decorate]
    struct Local {}

    #[tokio::test]
    async fn test_service() {
        let m = MongoStore::new("mongodb://10.200.100.200:27017")
            .await
            .unwrap();
        let s = Service::<Local, MongoStore>::new(
            "base".to_string(),
            "local".to_string(),
            Stores::new(m),
        );

        let q = query!();
        if let Ok(v) = s.list(q.clone()).await {
            println!("{:?}", v);
        } else {
            panic!("test error")
        }

        // if let Ok(v) = s.get(q.clone()).await {
        //     println!("{:?}", v);
        //     return;
        // } else {
        //     panic!("test error")
        // }
    }
}
