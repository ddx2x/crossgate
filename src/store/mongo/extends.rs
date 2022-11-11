use crate::utils::dict::compare_and_merge;

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

    type UpdateFuture<'a, T> = impl Future<Output = crate::Result<T>>
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

            let mut cursor = c.find(filter.get(), Some(opt)).await?;

            let mut items = vec![];
            while let Some(item) = cursor.try_next().await? {
                items.push(item);
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

            if let Some(value) = c.find_one(filter.get(), None).await? {
                return Ok(value);
            }

            return Err(StoreError::DataNotFound.into());
        };
        block
    }

    fn update_any_type<'r, T>(&'r self, t: T, q: Condition<F>) -> Self::UpdateFuture<'r, T>
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
}
