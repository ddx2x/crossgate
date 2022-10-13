use bson::Document;
use futures::{Future, TryStreamExt};

use mongodb::options::FindOptions;

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

    type RemoveFuture<'a,T> = impl Future<Output = crate::Result<()>>
    where
        Self: 'a,
        T: MongoDbModel;

    fn list_any_type<'r, T>(&'r self, q: Condition<F>) -> Self::ListFuture<'r, T>
    where
        T: MongoDbModel,
    {
        let block = async move {
            let Condition {
                db,
                table,
                filter,
                page,
                page_size,
                fields,
                sorts,
                ..
            } = q;

            let c = self.client.database(&db).collection::<T>(&table);

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

            if fields.len() > 0 {
                let mut doc = Document::new();
                for s in fields {
                    doc.insert(s.clone(), 1);
                }
                opt.projection = Some(doc);
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
        };

        block
    }

    fn save_any_type<'r, T>(&'r self, t: T, q: Condition<F>) -> Self::SaveFuture<'r, T>
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

    fn delete_any_type<'r, T>(&'r self, q: Condition<F>) -> Self::RemoveFuture<'r, T>
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

    fn get_any_type<'r, T>(&'r self, q: Condition<F>) -> Self::GetFuture<'r, T>
    where
        T: MongoDbModel,
    {
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
}
