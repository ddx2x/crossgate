use crate::store::mongo::matchs::matchs;
use crate::store::mongo_extends::MongoStorageOpExtends;
use crate::store::{current_time_sess, Event};
use crate::utils::dict::{from_unstructed_to_type, from_value_to_unstructed, get, value_to_map};
use crate::utils::Unstructed;
use crate::{store::mongo_extends::MongoStorageAggregationExtends, utils::dict::compare_and_merge};
use bson::{doc, Bson, Document, Uuid};
use condition::{parse, Value};
use futures::{Future, TryStreamExt};
use mongodb::options::{AggregateOptions, FindOneOptions, UpdateOptions};
use mongodb::{
    change_stream::event::{ChangeStreamEvent, OperationType},
    options::{ChangeStreamOptions, FindOptions, FullDocumentType},
};
use serde_json::Value as SerdeValue;

use tokio::sync::mpsc::Receiver;

use crate::store::{
    mongo_extends::{MongoDbModel, MongoStorageExtends},
    Condition, Filter, Result, StoreError,
};

use super::{uuid, GetFilter, MongoStore};

impl<F> MongoStorageExtends<F> for MongoStore
where
    F: Filter + GetFilter,
{
    type ListFuture<'a,T> = impl Future<Output = Result<Vec<T>>>
    where
        Self: 'a,
        T: MongoDbModel;

    fn list_any_type<'r, T>(self, q: Condition<F>) -> Self::ListFuture<'r, T>
    where
        T: MongoDbModel,
    {
        async move {
            let Condition {
                db,
                table,
                filter,
                page,
                size: page_size,
                fields,
                sorts,
                pageable,
                ..
            } = q;

            let c = self.client.database(&db).collection::<T>(&table);

            let mut opt = FindOptions::builder().build();

            if pageable {
                opt.skip = Some((page * page_size) as u64);
                opt.limit = Some(page_size as i64);
            }

            if sorts.len() > 0 {
                let mut doc = Document::new();
                for s in sorts {
                    match s.order {
                        crate::store::condition::SortDirection::Ascending => doc.insert(s.field, 1),
                        crate::store::condition::SortDirection::Descending => {
                            doc.insert(s.field, -1)
                        }
                    };
                }
                opt.sort = Some(doc);
            }

            if fields.len() > 0 {
                let mut doc = Document::new();
                for s in fields {
                    doc.insert(s.clone(), 1);
                }
                opt.projection = Some(doc);
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
        }
    }

    type SaveFuture<'a,T> =  impl Future<Output = Result<Option<T>>>
    where
        Self: 'a,
        T: MongoDbModel;
    fn save_any_type<'r, T>(self, t: T, q: Condition<F>) -> Self::SaveFuture<'r, T>
    where
        T: MongoDbModel,
    {
        let Condition { db, table, .. } = q;
        let c = self.collection::<T>(&db, &table);
        let block = async move {
            let mut value =
                from_value_to_unstructed(&t).map_err(|e| StoreError::OtherError(e.to_string()))?;
            if let SerdeValue::Null = value.get("_id") {
                value.set("_id", &SerdeValue::String(uuid()));
            }
            let t = from_unstructed_to_type::<T>(value)
                .map_err(|e| StoreError::OtherError(e.to_string()))?;

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

    type ApplyFuture<'a, T> = impl Future<Output = Result<T>>
    where
        Self: 'a,
        T: MongoDbModel;

    fn apply_any_type<'r, T>(self, t: T, q: Condition<F>) -> Self::ApplyFuture<'r, T>
    where
        T: MongoDbModel,
    {
        let Condition {
            db,
            table,
            filter,
            fields,
            ..
        } = q;

        let c = self.collection::<T>(&db, &table);
        let mut t = t;

        let block = async move {
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

            if let Ok(update) = compare_and_merge(&mut old.unwrap(), &mut t, fields) {
                let _ = c
                    .replace_one(filter, &update, None)
                    .await
                    .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
                return Ok(update);
            }

            return Ok(t);
        };

        block
    }

    type RemoveFuture<'a,T> = impl Future<Output = Result<()>>
    where
        Self: 'a,
        T: MongoDbModel;
    fn delete_any_type<'r, T>(self, q: Condition<F>) -> Self::RemoveFuture<'r, T>
    where
        T: MongoDbModel,
    {
        let Condition {
            db, table, filter, ..
        } = q;

        let c = self.collection::<T>(&db, &table);

        let block = async move {
            let _ = c
                .delete_many(filter.get_doc(), None)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
            Ok(())
        };

        block
    }

    type GetFuture<'a, T> = impl Future<Output = Result<T>>
    where
        Self: 'a,
        T: MongoDbModel;
    fn get_any_type<'r, T>(self, q: Condition<F>) -> Self::GetFuture<'r, T>
    where
        T: MongoDbModel,
    {
        let block = async move {
            let Condition {
                db,
                table,
                fields,
                filter,
                ..
            } = q;
            let c = self.collection::<T>(&db, &table);

            let mut opt = FindOneOptions::builder().build();

            if fields.len() > 0 {
                let mut doc = Document::new();
                for s in fields {
                    doc.insert(s.clone(), 1);
                }
                opt.projection = Some(doc);
            }

            if let Some(value) = c
                .find_one(filter.get_doc(), Some(opt))
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?
            {
                return Ok(value);
            }

            return Err(StoreError::DataNotFound.into());
        };

        block
    }

    type StreamFuture<'a, T> = impl Future<Output = Result<Receiver<Event<T>>>>
    where
        Self: 'a,
        T: MongoDbModel+ 'static;

    fn watch_any_type<'r, T>(
        self,
        ctx: crate::store::Context,
        q: Condition<F>,
    ) -> Self::StreamFuture<'r, T>
    where
        T: MongoDbModel,
    {
        let client = self.client.clone();
        let Condition {
            filter, db, table, ..
        } = q;

        let block = async move {
            let (tx, rx) = tokio::sync::mpsc::channel(4);

            let (filter_doc, filter_src) = filter.get();

            let collection = client.database(&db).collection::<T>(&table);
            let mut cursor = collection
                .find(filter_doc, None)
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
                            OperationType::Insert => {
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
                            OperationType::Update | OperationType::Replace => {
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
                            OperationType::Delete => {
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
        };

        block
    }

    type CountFuture<'a, T> = impl Future<Output = Result<u64>>
    where
        Self: 'a,
        T: MongoDbModel;
    fn count<'r, T>(self, q: Condition<F>) -> Self::CountFuture<'r, T>
    where
        T: MongoDbModel,
    {
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

    type UpdateFuture<'a, T> = impl Future<Output = Result<Option<T>>>
    where
        Self: 'a,
        T: MongoDbModel;

    fn update_any_type<'r, T>(self, t: T, q: Condition<F>) -> Self::UpdateFuture<'r, T>
    where
        T: MongoDbModel,
    {
        let Condition {
            db,
            table,
            filter,
            fields,
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
                        .map_err(|e| StoreError::ConnectionError(e.to_string()))?,
                );
            }

            update.insert("version", Bson::Int64(current_time_sess() as i64));

            let filter = filter.get_doc();
            let _ = c
                .update_one(filter.clone(), doc! {"$set":update}, options)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?;

            Ok(c.find_one(filter, None)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?)
        }
    }
}

impl MongoStorageAggregationExtends for MongoStore {
    type AggregationListFuture<'a, T> =  impl Future<Output = Result<Vec<T>>>
    where
        Self: 'a,
        T: MongoDbModel;

    fn aggregate<'r, T>(
        self,
        db: String,
        table: String,
        q: Vec<Document>,
    ) -> Self::AggregationListFuture<'r, T>
    where
        T: MongoDbModel,
    {
        let client = self.client.clone();

        let block = async move {
            let mut rs: Vec<T> = vec![];
            let options = Some(
                AggregateOptions::builder()
                    .allow_disk_use(Some(true))
                    .build(),
            );

            let mut cursor = client
                .database(&db)
                .collection::<T>(&table)
                .aggregate(q, options)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?;

            while let Some(item) = cursor
                .try_next()
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?
            {
                rs.push(
                    bson::from_document(item).map_err(|e| StoreError::OtherError(e.to_string()))?,
                );
            }

            Ok(rs)
        };

        block
    }
}

impl<F> MongoStorageOpExtends<F> for MongoStore
where
    F: Filter + GetFilter,
{
    type IncrFuture<'a, T> = impl Future<Output = Result<Option<T>>>
    where
        Self: 'a,
        T: MongoDbModel;

    fn incr<'r, T>(self, kv_pairs: &'r [(&'r str, i64)], q: Condition<F>) -> Self::IncrFuture<'r, T>
    where
        T: MongoDbModel,
    {
        let Condition {
            db, table, filter, ..
        } = q;

        let c = self.collection::<T>(&db, &table);

        let options = UpdateOptions::builder().upsert(false).build();
        let mut update = doc! {};
        update.insert("version", Bson::Int64(current_time_sess() as i64));

        let mut incr = doc! {};

        for (key, val) in kv_pairs {
            incr.insert(key.to_string(), val);
        }

        let block = async move {
            let filter = filter.get_doc();

            let _ = c
                .update_one(filter.clone(), doc! {"$inc":incr,"$set":update}, options)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?;

            Ok(c.find_one(filter, None)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?)
        };

        block
    }

    type BatchRemoveFuture<'a> = impl Future<Output = Result<u64>>
    where
        Self: 'a;

    fn batch_remove<'r>(self, q: Condition<F>) -> Self::BatchRemoveFuture<'r> {
        let Condition {
            db, table, filter, ..
        } = q;

        let c = self.collection::<Unstructed>(&db, &table);

        let block = async move {
            Ok(c.delete_many(filter.get_doc(), None)
                .await
                .map_err(|e| StoreError::ConnectionError(e.to_string()))?
                .deleted_count)
        };

        block
    }
}
