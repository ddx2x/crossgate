use super::Context;
use super::{condition::Condition, Query, StoreError, Stroage, Value};
use crate::object::Object;

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
            Err(e) => {
                log::error!("new MongoStore error: {:?}", e);
                Err(StoreError::ConnectionError)
            }
        }
    }
}

impl MongoStore {
    fn collection<T>(&self, q: Query<&str, Value>) -> Result<mongodb::Collection<T>, StoreError> {
        let condition = Condition::parse(q);

        Ok(self
            .client
            .database(condition.db)
            .collection::<T>(condition.table))
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
            let c = if let Ok(c) = self.collection::<T>(q) {
                c
            } else {
                return Err(StoreError::ConnectionError);
            };

            let mut cursor = if let Ok(cursor) = c.find(None, None).await {
                cursor
            } else {
                return Err(StoreError::ConnectionError);
            };

            let mut items = vec![];
            while let Some(item) = cursor.try_next().await.unwrap() {
                items.push(item)
            }
            Ok(items)
        };
        block
    }

    fn get<'r>(&'r self, q: Query<&'r str, Value<'r>>) -> Self::Future<'r> {
        let block = async move {
            let c = self.collection::<T>(q).unwrap();
            if let Some(r) = c.find_one(None, None).await.unwrap() {
                Ok(r)
            } else {
                Err(StoreError::DataNotFound.into())
            }
        };
        block
    }

    fn watch<'r>(&'r self, ctx: Context, db:String, table: String,q: Query<&str, Value<'_>>,) -> Self::StreamFuture<'r> {
        let client = self.client.clone();
        async move {
            let oplog =
                oplog::subscribe::<T>(ctx, client, &db, &table, None)
                    .unwrap();
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
