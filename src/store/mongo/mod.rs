mod filter;
pub use filter::MongoFilter;

use super::condition::Condition;
use super::{Context, Filter};
use super::{StoreError, Stroage};
use crate::object::Object;

use bson::oid::ObjectId;
use bson::Document;

use futures::{Future, TryStreamExt};
use mongodb::options::FindOptions;
use std::fmt::Debug;

use mongodb::Client;

use serde::de::DeserializeOwned;
use serde::Serialize;

use tokio::sync::mpsc::Receiver;

pub trait GetFilter {
    fn get(self) -> Document;
}

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
}

impl<T, F> Stroage<T, F> for MongoStore
where
    T: Object + DeserializeOwned + Serialize + Unpin + Debug,
    F: Filter + GetFilter,
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

    fn list<'r>(&'r self, q: Condition<F>) -> Self::ListFuture<'r> {
        let block = async move {
            let Condition {
                db,
                table,
                filter,
                page,
                page_size,
                sorts: sort,
            } = q;

            let c = self.collection::<T>(&db, &table);

            let mut opt = FindOptions::builder().build();

            if q.page != 0 {
                opt.skip = Some((q.page * q.page_size) as u64);
                opt.limit = Some(q.page_size as i64);
            }

            // let sorts = q.sorts();
            // if sorts.len() > 0 {
            //     let mut doc = Document::new();
            //     for s in sorts {
            //         doc.insert(s.clone(), 1);
            //     }
            //     opt.sort = Some(doc);
            // }

            let mut cursor = match c.find(filter.get(), Some(opt)).await {
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

    fn get<'r>(&'r self, q: Condition<F>) -> Self::GetFuture<'r> {
        let block = async move {
            let Condition {
                db, table, filter, ..
            } = q;
            let c = self.collection::<T>(&db, &table);

            match c.find_one(filter.get(), None).await {
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
        q: Condition<F>,
    ) -> Self::StreamFuture<'r> {
        let client = self.client.clone();
        let Condition { filter, .. } = q;
        let block = async move {
            let oplog =
                oplog::subscribe::<T>(ctx, client, &db, &table, Some(filter.get())).unwrap();
            oplog
        };

        block
    }

    fn save<'r>(&'r self, t: T, q: Condition<F>) -> Self::SaveFuture<'r> {
        let Condition { db, table, .. } = q;
        let c = self.collection::<T>(&db, &table);
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

    fn delete<'r>(&'r self, q: Condition<F>) -> Self::RemoveFuture<'r> {
        let Condition {
            db, table, filter, ..
        } = q;

        let c = self.collection::<T>(&db, &table);

        let block = async move {
            let _ = c.delete_many(filter.get(), None).await?;
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
