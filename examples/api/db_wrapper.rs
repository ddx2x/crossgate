use std::env;

use crossgate::store::MongoStore;

use once_cell::sync::OnceCell;

static MONGO_STORE: OnceCell<MongoStore> = OnceCell::new();

async fn _create_mongo_store() {
    let database_url = env::var("MONGO_URL").expect("MONGO_URL must be set");
    let store = MongoStore::new(&database_url)
        .await
        .expect("Mongo global must set success");

    MONGO_STORE
        .set(store)
        .expect("Mongo global must set success")
}

#[inline]
pub async fn get_mongo_store() -> &'static MongoStore {
    // Safety: tt is already set when the program is initialized
    if MONGO_STORE.get().is_none() {
        _create_mongo_store().await;
    }
    unsafe { MONGO_STORE.get_unchecked() }
}
