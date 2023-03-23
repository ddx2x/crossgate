mod filter;
use condition::parse;
pub use filter::MongoFilter;
use mongodb::change_stream::event::ChangeStreamEvent;
mod extends;
use super::condition::Condition;
use super::{current_time_sess, Context};
use super::{Event, Filter};
use super::{Storage, StoreError};
use crate::object::Object;

use crate::store::mongo::matchs::matchs;
use crate::store::Result;
use crate::utils::dict::{compare_and_merge, from_value_to_unstructed, get, value_to_map};
use crate::utils::{matchs, Unstructed};

use bson::oid::ObjectId;
use bson::{doc, Bson, Document};

use futures::{Future, TryStreamExt};
use mongodb::options::{ChangeStreamOptions, FindOptions, FullDocumentType, UpdateOptions};
use mongodb::{change_stream, Client};
use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

use tokio::sync::mpsc::Receiver;

pub fn new_mongo_condition() -> Condition<MongoFilter> {
    Condition::new(MongoFilter(doc! {}, "".to_string()))
}

fn uuid() -> String {
    ObjectId::new().to_string()
}

pub trait GetFilter {
    fn get_doc(self) -> Document;
    fn get_src(self) -> String;
    fn get(&self) -> (Document, String);
}

#[derive(Debug, Clone)]
pub struct MongoStore {
    client: Client,
}

impl MongoStore {
    pub async fn new(uri: &str) -> Result<Self> {
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
    type ListFuture<'a> = impl Future<Output = Result<Vec<T>>>
    where
        Self: 'a;
    fn list<'r>(self, q: Condition<F>) -> Self::ListFuture<'r> {
        let block = async move {
            let Condition {
                db,
                table,
                filter,
                page,
                size: page_size,
                sorts,
                pageable,
                ..
            } = q;

            let c = self.collection::<T>(&db, &table);

            let mut opt = FindOptions::builder().build();

            if pageable {
                opt.skip = Some((page * page_size) as u64);
                opt.limit = Some(page_size as i64);
            }

            if sorts.len() > 0 {
                let mut doc = Document::new();
                for s in sorts {
                    match s.order {
                        super::condition::SortDirection::Ascending => doc.insert(s.field, 1),
                        super::condition::SortDirection::Descending => doc.insert(s.field, -1),
                    };
                }
                opt.sort = Some(doc);
            }

            let mut cursor = c
                .find(filter.get_doc(), Some(opt))
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?;

            let mut items = vec![];

            while let Some(item) = cursor
                .try_next()
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?
            {
                items.push(item);
            }

            Ok(items)
        };

        block
    }

    type GetFuture<'a> = impl Future<Output =  Result<T>>
    where
        Self: 'a;
    fn get<'r>(self, q: Condition<F>) -> Self::GetFuture<'r> {
        let block = async move {
            let Condition {
                db, table, filter, ..
            } = q;
            let c = self.collection::<T>(&db, &table);

            if let Some(value) = c
                .find_one(filter.get_doc(), None)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?
            {
                return Ok(value);
            }

            Err(StoreError::DataNotFound.into())
        };

        block
    }

    type StreamFuture<'a> = impl Future<Output = Result<Receiver<Event<T>>>>
    where
        Self: 'a;
    fn watch<'r>(self, ctx: Context, q: Condition<F>) -> Self::StreamFuture<'r> {
        let client = self.client.clone();
        let Condition {
            filter, db, table, ..
        } = q;

        async move {
            let (tx, rx) = tokio::sync::mpsc::channel(1);

            let (filter_doc, filter_src) = filter.get();
            let collection = client.database(&db).collection::<T>(&table);
            let mut cursor = collection
                .find(filter_doc.clone(), None)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?;

            let options = ChangeStreamOptions::builder()
                .full_document(Some(FullDocumentType::UpdateLookup))
                .build();

            let mut stream = collection
                .watch(None, options)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?;

            let _matchs = move |item: &T| -> bool {
                if filter_src.eq("") {
                    return true;
                }
                if let Ok(v) = from_value_to_unstructed(item) {
                    if let Ok(r) = matchs(&mut vec![v], parse(&filter_src).unwrap()) {
                        return r.len() == 1;
                    }
                }
                return false;
            };

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

                                let item = full_document.unwrap();

                                if !_matchs(&item) {
                                    continue;
                                }

                                if let Err(e) = tx.send(Event::Added(item)).await {
                                    log::error!("{:?}", e.to_string());
                                    break;
                                }
                            }
                            change_stream::event::OperationType::Update
                            | change_stream::event::OperationType::Replace => {
                                if full_document.is_none() {
                                    break;
                                }
                                let item = full_document.unwrap();

                                if !_matchs(&item) {
                                    continue;
                                }
                                if let Err(e) = tx.send(Event::Updated(item)).await {
                                    log::error!("{:?}", e.to_string());
                                    break;
                                }
                            }
                            change_stream::event::OperationType::Delete => {
                                if document_key.is_none() {
                                    break;
                                }
                                match mongodb::bson::from_document::<Unstructed>(
                                    document_key.unwrap(),
                                ) {
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

    type SaveFuture<'a> = impl Future<Output =  Result<Option<T>>>
    where
        Self: 'a;

    fn save<'r>(self, t: T, q: Condition<F>) -> Self::SaveFuture<'r> {
        let Condition { db, table, .. } = q;
        let c = self.collection::<T>(&db, &table);
        let block = async move {
            let mut t = t;

            t.uid().is_empty().then(|| t.set_uid(&uuid()));

            let insert_one_result = c
                .insert_one(t, None)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?;

            Ok(
                c.find_one(doc! {"_id":insert_one_result.inserted_id.as_str()}, None)
                    .await
                    .map_err(|e| StoreError::ConnectionError(e.to_string()))?,
            )
        };

        block
    }

    type ApplyFuture<'a> = impl Future<Output =  Result<T>>
        where
            Self: 'a;
    fn apply<'r>(self, t: T, q: Condition<F>) -> Self::ApplyFuture<'r> {
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
            let filter = filter.get_doc();
            let old = c
                .find_one(filter.clone(), None)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?;

            if old.is_none() {
                let _ = c
                    .insert_one(&t, None)
                    .await
                    .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
                return Ok(t);
            }

            if let Ok(mut update) = compare_and_merge(&mut old.unwrap(), &mut t, fields) {
                update.set_version(current_time_sess());

                let _ = c
                    .replace_one(filter, &update, None)
                    .await
                    .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
                return Ok(update);
            }

            return Ok(t);
        }
    }

    type RemoveFuture<'a>= impl Future<Output =  Result<()>>
    where
        Self: 'a;
    fn delete<'r>(self, q: Condition<F>) -> Self::RemoveFuture<'r> {
        let Condition {
            db, table, filter, ..
        } = q;

        let c = self.collection::<T>(&db, &table);

        async move {
            let _ = c
                .delete_many(filter.get_doc(), None)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
            Ok(())
        }
    }

    type CountFuture<'a>= impl Future<Output = Result<u64>>
    where
        Self: 'a;

    fn count<'r>(self, q: Condition<F>) -> Self::CountFuture<'r> {
        let block = async move {
            let Condition {
                db, table, filter, ..
            } = q;
            let c = self.collection::<T>(&db, &table);

            Ok(c.count_documents(filter.get_doc(), None)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?)
        };
        block
    }

    type UpdateFuture<'a> = impl Future<Output =  Result<Option<T>>>
        where
            Self: 'a;
    fn update<'r>(self, t: T, q: Condition<F>) -> Self::UpdateFuture<'r> {
        let Condition {
            db,
            table,
            filter,
            fields,
            update_version,
            ..
        } = q;

        let c = self.collection::<T>(&db, &table);

        async move {
            let options = UpdateOptions::builder().upsert(false).build();
            let mut update = doc! {};
            let mut map = value_to_map(&t).map_err(|e| StoreError::OtherError(e.to_string()))?;
            for field in fields {
                update.insert(
                    field.clone(),
                    bson::to_bson(&get(&mut map, &field))
                        .map_err(|e| StoreError::OtherError(e.to_string()))?,
                );
            }
            if update_version {
                update.insert("version", Bson::Int64(current_time_sess() as i64));
            }

            let filter = filter.get_doc();
            let _ = c
                .update_one(filter.clone(), doc! {"$set":update}, options)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?;

            let rs = c
                .find_one(filter, None)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
            Ok(rs)
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
