mod login;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let listen_addr =
        ::std::env::var("API_SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    crossgate_rs::micro::run_api_server(listen_addr, &[login::handle], None).await
}
