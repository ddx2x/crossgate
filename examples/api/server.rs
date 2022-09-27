use async_stream::stream;
use axum::{
    extract::TypedHeader,
    response::sse::{Event, KeepAlive, Sse},
    response::{IntoResponse, Response},
    routing::{get, get_service},
    Extension, Router,
};
use tokio_context::context::Context;

use futures::{future::BoxFuture, stream::Stream};
use std::{convert::Infallible, net::SocketAddr, time::Duration};

use crate::{
    base::{Base, Local},
    db_wrapper::get_mongo_store,
};

async fn hello() -> &'static str {
    "base"
}

async fn list_local(Extension(base): Extension<Base>) -> impl IntoResponse {
    axum::Json(base.list().await)
}
async fn get_local(Extension(base): Extension<Base>) -> impl IntoResponse {
    axum::Json(base.get("abc").await)
}

async fn watch(
    Extension(base): Extension<Base>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {
        let (ctx, h) = Context::new();
        let mut r = base.watch(ctx).await;
        while let Some(item) = r.recv().await {
            if let oplog::Event::Error(e) = item {
                log::error!("error {:?}",e);
                break;
            }
            yield Ok(Event::default().data(format!("{}", item)));
        }
        h.cancel();
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive"),
    )
}

pub fn run<'a>(addr: &'a SocketAddr) -> BoxFuture<'a, ()> {
    let block = async move {
        let base = crossgate_rs::micro::make_service(crate::base::Base::create(
            addr,
            get_mongo_store().await,
        ))
        .await;

        let app = Router::new()
            .route("/base/watch", get(watch))
            .route("/base/locals", get(list_local))
            .route("/base/local", get(get_local))
            .route("/base", get(hello))
            .layer(Extension(base));

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    };

    Box::pin(block)
}
