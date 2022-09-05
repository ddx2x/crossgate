use std::net::SocketAddr;
mod base;
mod db_wrapper;
mod server;

use crossbeam::sync::WaitGroup;
use tokio;
use tokio_context::context::Context;


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let listen_addr0 =
        ::std::env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    log::info!("listen_addr0: {}", listen_addr0);

    let (ctx, handle) = Context::new();
    let wg = WaitGroup::new();

    crossgate_rs::plugin::init_plugin(
        ctx,
        wg.clone(),
        crossgate_rs::plugin::ServiceType::WebService,
        crossgate_rs::plugin::PluginType::Mongodb,
    )
    .await;

    tokio::select! {
        _ = server::run(listen_addr0.parse::<SocketAddr>().unwrap()) => {},
        _ = tokio::signal::ctrl_c() => {
            handle.cancel();
            wg.wait();
        },
    }
}
