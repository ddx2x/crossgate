use super::Context;
use super::{condition::Condition, Query, StoreError, Stroage, Value};
use crate::object::Object;

use bson::oid::ObjectId;
use bson::{doc, Document};

use futures::{Future, TryStreamExt};
use mongodb::options::{FindOneOptions, FindOptions};
use std::fmt::Debug;

use mongodb::Client;

use serde::de::DeserializeOwned;
use serde::Serialize;

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
            Err(e) => Err(StoreError::ConnectionError(e.to_string())),
        }
    }
}

impl MongoStore {
    fn collection<'a, T>(&self, db: &str, table: &str) -> mongodb::Collection<T> {
        self.client.database(db).collection::<T>(table)
    }

    fn find_option(&self, condition: &Condition) -> FindOptions {
        let mut opt = FindOptions::builder().build();

        if condition.page != 0 {
            opt.skip = Some((condition.page * condition.page_size) as u64);
            opt.limit = Some(condition.page_size as i64);
        }

        if condition.sort.len() > 0 {
            let mut doc = Document::new();
            for s in &condition.sort {
                doc.insert(s, 1);
            }
            opt.sort = Some(doc);
        }

        opt
    }

    fn condition<'a>(
        &self,
        q: Query<&'a str, Value<'a>>,
    ) -> (&'a str, &'a str, Document, FindOptions) {
        let condition = Condition::parse(q);

        let find_options = self.find_option(&condition);

        let mut doc = doc! {};
        for (mut k, v) in condition.other {
            if k == crate::store::UID {
                k = "_id".to_string(); //rewrite default uid to _id
            }
            match v {
                Value::String(s) => doc.insert(k, s),
                Value::Number(n) => doc.insert(k, n),
                Value::Boolean(b) => doc.insert(k, b),
                Value::Array(_) => todo!(),
                Value::Null => todo!(),
            };
        }
        (condition.db, condition.table, doc, find_options)
    }
}

impl<T> Stroage<T> for MongoStore
where
    T: Object + DeserializeOwned + Serialize + Unpin + Debug,
{
    type ListFuture<'a> = impl Future<Output = crate::Result<Vec<T>>>
    where
        Self: 'a;

    type GetFuture<'a> = impl Future<Output =  crate::Result<T>> 
    where
        Self: 'a;

    type SaveFuture<'a> = impl Future<Output =  crate::Result<()>> 
    where
        Self: 'a;

    type RemoveFuture<'a>= impl Future<Output =  crate::Result<()>> 
    where
        Self: 'a;

    type StreamFuture<'a> = impl Future<Output = Receiver<oplog::Event<T>>>
    where
        Self: 'a;

    fn list<'r>(&'r self, q: Query<&'r str, Value<'r>>) -> Self::ListFuture<'r> {
        let block = async move {
            let (db, table, filter, find_opt) = self.condition(q);
            let c = self.collection::<T>(db, table);

            let mut cursor = match c.find(filter, Some(find_opt)).await {
                Ok(c) => c,
                Err(e) => {
                    return Err(anyhow::format_err!(
                        "mongodb find error: {:?}",
                        StoreError::Other(Box::new(e))
                    ))
                }
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
                    Err(e) => {
                        return Err(anyhow::format_err!(
                            "mongodb find cursor error: {:?}",
                            StoreError::Other(Box::new(e))
                        ))
                    }
                }
            }

            Ok(items)
        };
        block
    }

    fn get<'r>(&'r self, q: Query<&'r str, Value<'r>>) -> Self::GetFuture<'r> {
        let block = async move {
            let (db, table, filter, _) = self.condition(q);
            let c = self.collection::<T>(db, table);

            match c.find_one(filter, None).await {
                Ok(value) => {
                    if let Some(value) = value {
                        return Ok(value);
                    } else {
                        return Err(StoreError::DataNotFound.into());
                    }
                }
                Err(e) => {
                    return Err(anyhow::format_err!(
                        "mongodb get error: {:?}",
                        StoreError::Other(Box::new(e))
                    ))
                }
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
        let (_, _, filter, _) = self.condition(q);
        let block = async move {
            let oplog = oplog::subscribe::<T>(ctx, client, &db, &table, Some(filter)).unwrap();
            oplog
        };

        block
    }

    fn save<'r>(&'r self, t: T, q: Query<&str, Value<'r>>) -> Self::SaveFuture<'r> {
        let (db, table, _, _) = self.condition(q);
        let c = self.collection::<T>(db, table);
        let block = async move {
            let mut t = t;
            if t.uid().len() == 0 || t.uid() == "" {
                t.generate(|| -> String { ObjectId::new().to_string() });
            }
            let _ = c.insert_one(t, None).await?;
            Ok(())
        };

        block
    }

    fn delete<'r>(&'r self, q: Query<&str, Value<'r>>) -> Self::RemoveFuture<'r> {
        let (db, table, filter, _) = self.condition(q);
        let c = self.collection::<T>(db, table);

        let block = async move {
            let _ = c.delete_many(filter, None).await?;
            Ok(())
        };

        block
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
