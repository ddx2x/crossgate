use super::Context;
use super::{condition::Condition, Query, StoreError, Stroage, Value};
use crate::object::Object;

use bson::{doc, Document};
use futures::TryStreamExt;
use std::fmt::Debug;
use std::future::Future;

use mongodb::Client;

use serde::de::DeserializeOwned;
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
pub struct MongoStore {
    client: Client,
}

impl MongoStore {
    pub async fn new(uri: &str) -> Result<Self, StoreError> {
        match mongodb::options::ClientOptions::parse_with_resolver_config(
            &uri,
            mongodb::options::ResolverConfig::cloudflare(),
        )
        .await
        {
            Ok(options) => {
                let client = Client::with_options(options).unwrap();
                Ok(Self { client })
            }
            Err(e) => Err(StoreError::ConnectionError(e.to_string()))
        }
    }
}

impl MongoStore {
    fn collection<'a, T>(&self, db: &str, table: &str) -> mongodb::Collection<T> {
        self.client.database(db).collection::<T>(table)
    }

    fn condition<'a>(&self, q: Query<&'a str, Value<'a>>) -> (&'a str, &'a str, Document) {
        let condition = Condition::parse(q);
        let mut doc = doc! {};
        for (k, v) in condition.other {
            match v {
                Value::String(s) => doc.insert(k, s),
                Value::Number(n) => doc.insert(k, n),
                Value::Boolean(b) => doc.insert(k, b),
                Value::Array(a) => todo!(),
                Value::Pair(_) => todo!(),
                Value::Null => todo!(),
            };
        }
        (condition.db, condition.table, doc)
    }
}

impl<T> Stroage<T> for MongoStore
where
    T: Object + DeserializeOwned + Unpin + Debug,
{
    type VectorFuture<'a> = impl Future<Output = Result<Vec<T>, StoreError>>
    where
        Self: 'a;

    type Future<'a> = impl Future<Output = Result<T, StoreError>> 
    where
        Self: 'a;

    type StreamFuture<'a> = impl Future<Output = Receiver<oplog::Event<T>>>
    where
        Self: 'a;

    fn list<'r>(&'r self, q: Query<&'r str, Value<'r>>) -> Self::VectorFuture<'r> {
        let block = async move {
            let (db, table, filter) = self.condition(q);
            let c = self.collection::<T>(db, table);

            let mut cursor = match c.find(filter, None).await {
                Ok(c) => c,
                Err(e) => return Err(StoreError::Other(Box::new(e))),
            };

            let mut items = vec![];
            loop {
                match cursor.try_next().await {
                    Ok(item) => {
                        if let Some(item) = item {
                            items.push(item);
                            continue;
                        }
                        break;
                    }
                    Err(e) => return Err(StoreError::Other(Box::new(e))),
                }
            }

            Ok(items)
        };
        block
    }

    fn get<'r>(&'r self, q: Query<&'r str, Value<'r>>) -> Self::Future<'r> {
        let block = async move {
            let (db, table, filter) = self.condition(q);
            let c = self.collection::<T>(db, table);

            match c.find_one(filter, None).await {
                Ok(value) => {
                    if let Some(value) = value {
                        return Ok(value);
                    } else {
                        return Err(StoreError::DataNotFound.into());
                    }
                }
                Err(e) => return Err(StoreError::Other(Box::new(e))),
            }
        };
        block
    }

    fn watch<'r>(
        &'r self,
        ctx: Context,
        db: String,
        table: String,
        q: Query<&str, Value<'_>>,
    ) -> Self::StreamFuture<'r> {
        let client = self.client.clone();
        async move {
            let oplog = oplog::subscribe::<T>(ctx, client, &db, &table, None).unwrap();
            oplog
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mongostore_test() {
        if let Ok(_) = MongoStore::new(r#"mongodb://localhost:27017"#).await {
        } else {
            panic!("test failed");
        }
    }
}
