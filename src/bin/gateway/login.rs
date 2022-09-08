use std::pin::Pin;

use futures::{future::BoxFuture, Future};
use hyper::{Body, Request, Response};

mod User {
    use crossgate::object::{decorate, Object};
    #[decorate]
    struct User {}
}

mod Role {
    use crossgate::object::{decorate, Object};

    #[decorate]
    struct Role {}
}

pub fn handle<'a>(
    r: &'a mut Request<Body>,
    w: &'a mut Response<Body>,
) -> BoxFuture<'a, crossgate_rs::micro::IntercepterType> {
    Box::pin(async move { crossgate_rs::micro::IntercepterType::Redirect })
}
