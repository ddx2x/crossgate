use std::net::SocketAddr;
mod base;
mod db_wrapper;
mod server;

use tokio;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let listen_addr =
        ::std::env::var("LISTEN_ADDRESS").unwrap_or_else(|_| get_address().unwrap().to_string());

    log::info!("listen_addr: {}", listen_addr);

    let addr = listen_addr.parse::<SocketAddr>().unwrap();

    crossgate_rs::micro::web_service_run(
        &addr,
        server::run,
        crossgate_rs::plugin::PluginType::Mongodb,
    )
    .await
}

pub fn get_address() -> anyhow::Result<SocketAddr> {
    use std::net::TcpListener;
    Ok(TcpListener::bind("0.0.0.0:0")?.local_addr()?)
}
