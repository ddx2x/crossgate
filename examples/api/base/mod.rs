pub mod gps;
pub mod local;
use std::net::SocketAddr;

use crossgate::store::Stores;
pub use local::Local;

use crossgate::service::{self, Service};
use crossgate::{query, store::MongoStore};
use tokio::sync::mpsc::Receiver;
use tokio_context::context::Context;

#[derive(Debug, Clone)]
pub struct Base {
    addr: SocketAddr,

    loc: service::Service<Local, MongoStore>,
    gps: service::Service<gps::Gps, MongoStore>,
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
    pub fn create(addr: SocketAddr, store: &MongoStore) -> Self {
        let base = Self {
            loc: Service::<Local, MongoStore>::new(
                "base".to_string(),
                "local".to_string(),
                Stores::new(store.clone()),
            ),
            gps: Service::<gps::Gps, MongoStore>::new(
                "base".to_string(),
                "gps_latest".to_string(),
                Stores::new(store.clone()),
            ),
            addr: addr,
        };
        base
    }

    pub async fn list(&self) -> Vec<Local> {
        if let Ok(rs) = self.loc.list(query!()).await {
            return rs;
        }
        vec![]
    }

    pub async fn get(&self) -> Local {
        self.loc.get(query!()).await.unwrap()
    }

    pub async fn watch(&self, ctx: Context) -> Receiver<oplog::Event<gps::Gps>> {
        self.gps.watch(ctx, query!()).await
    }
}
