use crate::{store::mongo_extends::MongoStorageAggregationExtends, utils::dict::compare_and_merge};

use crate::store::Event;
use bson::Document;
use futures::{Future, TryStreamExt};
use mongodb::options::AggregateOptions;
use mongodb::{
    change_stream::event::{ChangeStreamEvent, OperationType},
    options::{ChangeStreamOptions, FindOptions, FullDocumentType},
};

use tokio::sync::mpsc::Receiver;

use crate::store::{
    mongo_extends::{MongoDbModel, MongoStorageExtends},
    Condition, Filter, StoreError,
};

use super::{GetFilter, MongoStore};

impl<F> MongoStorageExtends<F> for MongoStore
where
    F: Filter + GetFilter,
{
    type ListFuture<'a,T> = impl Future<Output = crate::Result<Vec<T>>>
    where
        Self: 'a,
        T: MongoDbModel;

    type GetFuture<'a, T> = impl Future<Output = crate::Result<T>>
    where
        Self: 'a,
        T: MongoDbModel;

    type SaveFuture<'a,T> =  impl Future<Output = crate::Result<()>>
    where
        Self: 'a,
        T: MongoDbModel;

    type UpdateFuture<'a, T> = impl Future<Output = crate::Result<T>>
    where
        Self: 'a,
        T: MongoDbModel;

    type RemoveFuture<'a,T> = impl Future<Output = crate::Result<()>>
    where
        Self: 'a,
        T: MongoDbModel;

    type StreamFuture<'a, T> = impl Future<Output = crate::Result<Receiver<Event<T>>>>
    where
        Self: 'a,
        T: MongoDbModel+ 'static;

    type CountFuture<'a, T> = impl Future<Output = crate::Result<u64>>
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

            let mut cursor = c.find(filter.get(), Some(opt)).await?;

            let mut items = vec![];
            while let Some(item) = cursor.try_next().await? {
                items.push(item);
            }

            Ok(items)
        }
    }

    fn save_any_type<'r, T>(self, t: T, q: Condition<F>) -> Self::SaveFuture<'r, T>
    where
        T: MongoDbModel,
    {
        let Condition { db, table, .. } = q;
        let c = self.collection::<T>(&db, &table);
        let block = async move {
            let _ = c.insert_one(t, None).await?;
            Ok(())
        };

        block
    }

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
        let mut t = t;

        let block = async move {
            let filter = filter.get();
            let old = c.find_one(filter.clone(), None).await?;

            if old.is_none() {
                let _ = c.insert_one(&t, None).await?;
                return Ok(t);
            }

            if let Ok(update) = compare_and_merge(&mut old.unwrap(), &mut t, fields) {
                let _ = c.replace_one(filter, &update, None).await?;
                return Ok(update);
            }

            return Ok(t);
        };

        block
    }

    fn delete_any_type<'r, T>(self, q: Condition<F>) -> Self::RemoveFuture<'r, T>
    where
        T: MongoDbModel,
    {
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

    fn get_any_type<'r, T>(self, q: Condition<F>) -> Self::GetFuture<'r, T>
    where
        T: MongoDbModel,
    {
        let block = async move {
            let Condition {
                db, table, filter, ..
            } = q;
            let c = self.collection::<T>(&db, &table);

            if let Some(value) = c.find_one(filter.get(), None).await? {
                return Ok(value);
            }

            return Err(StoreError::DataNotFound.into());
        };

        block
    }

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
                            OperationType::Insert => {
                                if full_document.is_none() {
                                    break;
                                }
                                if let Err(e) = tx.send(Event::Added(full_document.unwrap())).await
                                {
                                    log::error!("{:?}", e.to_string());
                                    break;
                                }
                            }
                            OperationType::Update | OperationType::Replace => {
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
                            OperationType::Delete => {
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
        };

        block
    }

    fn count<'r, T>(self, q: Condition<F>) -> Self::CountFuture<'r, T>
    where
        T: MongoDbModel,
    {
        let block = async move {
            let Condition {
                db, table, filter, ..
            } = q;
            let c = self.collection::<T>(&db, &table);

            Ok(c.count_documents(filter.get(), None).await?)
        };

        block
    }
}

impl MongoStorageAggregationExtends for MongoStore {
    type AggregationListFuture<'a, T> =  impl Future<Output = crate::Result<Vec<T>>>
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
                .await?;

            while let Some(item) = cursor.try_next().await? {
                rs.push(bson::from_document(item)?);
            }

            Ok(rs)
        };

        block
    }
}
