use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    object::Object,
    store::{MongoFilter, MongoStore},
};

use super::Service;

pub struct MongoStoreService<T>(pub Service<T, MongoFilter, MongoStore>)
where
    T: Object + Serialize + Debug + DeserializeOwned + std::marker::Unpin;

impl<T> MongoStoreService<T>
where
    T: Object + Serialize + Debug + DeserializeOwned + std::marker::Unpin,
{
    pub fn new(db: &str, table: &str, store: MongoStore) -> MongoStoreService<T> {
        Self(Service::new(db.to_string(), table.to_string(), store))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{decorate, Object};

    #[decorate]
    struct Test {}

    #[tokio::test]
    async fn mongo_store_example() {
        if let Ok(store) = MongoStore::new(r#"mongodb://localhost:27017"#).await {
            let _ = MongoStoreService::<Test>::new("test", "test", store);
        } else {
            panic!("test failed");
        }
    }
}
