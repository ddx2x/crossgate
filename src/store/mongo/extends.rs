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
}
