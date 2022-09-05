#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
use tokio::{self, net::TcpListener, signal};

mod echo;
mod jt808;

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get()) // m1 8 个核个工作线程
        .enable_io() // 可在runtime中使用异步IO
        .enable_time() // 可在runtime中使用异步计时器(timer)
        .build() // 创建runtime
        .unwrap();

    let listen_addr0 =
        ::std::env::var("LISTEN_ADDRESS0").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listen_addr1 =
        ::std::env::var("LISTEN_ADDRESS1").unwrap_or_else(|_| "0.0.0.0:3001".to_string());

    rt.block_on(async move {
        let listener0 = TcpListener::bind(&listen_addr0).await.unwrap();
        let listener1 = TcpListener::bind(&listen_addr1).await.unwrap();
        log::info!(
            "stream server listening on {},{}",
            listen_addr0,
            listen_addr1
        );

        tokio::select! {  // go for { select {} }
        _ =  crossgate_rs::net::run(
            listener0,
            echo::EchoFrameHandle{},
            signal::ctrl_c(),
        ) => {},
        _ =  crossgate_rs::net::run(
            listener1,
            jt808::JT808Handle {},
            signal::ctrl_c(),
        ) => {},
        }
    });
}
