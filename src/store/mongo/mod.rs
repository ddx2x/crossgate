mod filter;
pub use filter::MongoFilter;
use mongodb::change_stream::event::ChangeStreamEvent;
mod extends;
use super::condition::Condition;
use super::{current_time_sess, Context};
use super::{Event, Filter};
use super::{Storage, StoreError};
use crate::object::Object;

use crate::utils::dict::compare_and_merge;

use bson::oid::ObjectId;
use bson::{doc, Document};

use futures::{Future, TryStreamExt};
use mongodb::options::{ChangeStreamOptions, FindOptions, FullDocumentType};
use mongodb::{change_stream, Client};
use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

use tokio::sync::mpsc::Receiver;

pub fn new_mongo_condition() -> Condition<MongoFilter> {
    Condition::new(MongoFilter(doc! {}))
}

fn uuid() -> String {
    ObjectId::new().to_string()
}

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

impl<T, F> Storage<T, F> for MongoStore
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

    type UpdateFuture<'a> = impl Future<Output =  crate::Result<T>>
        where
            Self: 'a;

    type RemoveFuture<'a>= impl Future<Output =  crate::Result<()>>
    where
        Self: 'a;

    type StreamFuture<'a> = impl Future<Output = crate::Result<Receiver<Event<T>>>>
    where
        Self: 'a;

    fn list<'r>(&'r self, q: Condition<F>) -> Self::ListFuture<'r> {
        async move {
            let Condition {
                db,
                table,
                filter,
                page,
                page_size,
                sorts,
                ..
            } = q;

            let c = self.collection::<T>(&db, &table);

            let mut opt = FindOptions::builder().build();

            if page != 0 {
                opt.skip = Some(((page - 1) * page_size) as u64);
                opt.limit = Some(page_size as i64);
            }

            if sorts.len() > 0 {
                let mut doc = Document::new();
                for s in sorts {
                    doc.insert(s.clone(), 1);
                }
                opt.sort = Some(doc);
            }

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
        }
    }

    fn get<'r>(&'r self, q: Condition<F>) -> Self::GetFuture<'r> {
        async move {
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
        }
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

        async move {
            let (tx, rx) = tokio::sync::mpsc::channel(4);

            let collection = client.database(&db).collection::<T>(&table);
            let mut cursor = collection.find(filter.get(), None).await?;

            let options = ChangeStreamOptions::builder()
                .full_document(Some(FullDocumentType::UpdateLookup))
                .build();

            let mut stream = collection.watch(None, options).await?;

            tokio::spawn(async move {
                let loop_block = async {
                    while let Ok(Some(item)) = cursor.try_next().await {
                        if let Err(e) = tx.send(Event::Added(item)).await {
                            log::error!("watch find send: {}", e.to_string());
                            break;
                        }
                    }

                    while let Ok(Some(evt)) = stream.try_next().await {
                        let ChangeStreamEvent::<T> {
                            operation_type,
                            full_document,
                            document_key,
                            ..
                        } = evt;

                        match operation_type {
                            change_stream::event::OperationType::Insert => {
                                if full_document.is_none() {
                                    break;
                                }
                                if let Err(e) = tx.send(Event::Added(full_document.unwrap())).await
                                {
                                    log::error!("{:?}", e.to_string());
                                    break;
                                }
                            }
                            change_stream::event::OperationType::Update
                            | change_stream::event::OperationType::Replace => {
                                if full_document.is_none() {
                                    break;
                                }
                                if let Err(e) =
                                    tx.send(Event::Updated(full_document.unwrap())).await
                                {
                                    log::error!("{:?}", e.to_string());
                                    break;
                                }
                            }
                            change_stream::event::OperationType::Delete => {
                                if document_key.is_none() {
                                    break;
                                }
                                match mongodb::bson::from_document::<T>(document_key.unwrap()) {
                                    Ok(doc) => {
                                        if let Err(e) = tx.send(Event::Deleted(doc)).await {
                                            log::error!("{:?}", e.to_string());
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("{:?}", e.to_string());
                                        break;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                };

                let mut ctx = ctx;
                tokio::select! {
                    _ = loop_block => {},
                    _ = ctx.done() => return,
                }
            });

            Ok(rx)
        }
    }

    fn save<'r>(&'r self, t: T, q: Condition<F>) -> Self::SaveFuture<'r> {
        let Condition { db, table, .. } = q;
        let c = self.collection::<T>(&db, &table);
        let block = async move {
            let mut t = t;
            t.uid().is_empty().then(|| t.set_uid(&uuid()));
            let _ = c.insert_one(t, None).await?;
            Ok(())
        };

        block
    }

    fn update<'r>(&'r self, t: T, q: Condition<F>) -> Self::UpdateFuture<'r> {
        let Condition {
            db,
            table,
            filter,
            fields,
            ..
        } = q;

        let c = self.collection::<T>(&db, &table);
        let mut t = t;

        async move {
            let filter = filter.get();
            let old = c.find_one(filter.clone(), None).await?;

            if old.is_none() {
                let _ = c.insert_one(&t, None).await?;
                return Ok(t);
            }

            if let Ok(mut update) = compare_and_merge(&mut old.unwrap(), &mut t, fields) {
                update.set_version(current_time_sess());

                let _ = c.replace_one(filter, &update, None).await?;
                return Ok(update);
            }

            return Ok(t);
        }
    }

    fn delete<'r>(&'r self, q: Condition<F>) -> Self::RemoveFuture<'r> {
        let Condition {
            db, table, filter, ..
        } = q;

        let c = self.collection::<T>(&db, &table);

        async move {
            let _ = c.delete_many(filter.get(), None).await?;
            Ok(())
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
