pub mod gps;
pub mod local;
use std::net::SocketAddr;

use bson::doc;
use crossgate::store::{Condition, Event, MongoFilter, Stores};
pub use local::Local;

use crossgate::service::{self, Service};
use crossgate::store::MongoStore;
use tokio::sync::mpsc::Receiver;
use tokio_context::context::Context;

#[derive(Debug, Clone)]
pub struct Base {
    addr: SocketAddr,

    loc: service::Service<Local, MongoFilter, MongoStore>,
    gps: service::Service<gps::Gps, MongoFilter, MongoStore>,
}

impl crossgate_rs::micro::Service for Base {
    fn name(&self) -> String {
        "base".to_owned()
    }
    fn addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Base {
    pub fn create(addr: &SocketAddr, store: &MongoStore) -> Self {
        let base = Self {
            loc: Service::<Local, MongoFilter, MongoStore>::new(
                "base".to_string(),
                "local".to_string(),
                Stores::new(store.clone()),
            ),
            gps: Service::<gps::Gps, MongoFilter, MongoStore>::new(
                "base".to_string(),
                "gps_latest".to_string(),
                Stores::new(store.clone()),
            ),
            addr: addr.clone(),
        };
        base
    }

    pub async fn list(&self) -> Vec<Local> {
        let mut cond = Condition::new(MongoFilter(doc! {}));
        // if let Err(e) = cond.wheres("status=1") {
        //     return vec![];
        // };
        if let Ok(rs) = self.loc.list(cond).await {
            return rs;
        }
        vec![]
    }

    pub async fn get(&self, name: &str) -> Local {
        let mut cond = Condition::new(MongoFilter(doc! {}));
        cond.wheres(&format!("name='{}'", name)).unwrap();
        self.loc.get(cond).await.unwrap()
    }

    pub async fn watch(&self, ctx: Context) -> Receiver<Event<Local>> {
        let mut cond = Condition::new(MongoFilter(doc! {}));
        cond.wheres("version>=1").unwrap();

        log::info!("{:?}", cond);
        self.loc.watch(ctx, cond).await
    }
}
