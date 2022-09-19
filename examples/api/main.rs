use std::net::SocketAddr;
mod base;
mod db_wrapper;
mod server;

use tokio;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let listen_addr0 =
        ::std::env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    log::info!("listen_addr0: {}", listen_addr0);

    let addr = listen_addr0.parse::<SocketAddr>().unwrap();
    
    crossgate_rs::micro::web_service_run(
        &addr,
        server::run,
        crossgate_rs::plugin::PluginType::Mongodb,
    )
    .await
}
