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

pub(crate) fn handle(req: &Request<Body>, w: &mut Response<Body>) -> crossgate_rs::micro::InterceptType {
    log::info!("handle request: {:?}", req);

    crossgate_rs::micro::InterceptType::Redirect
}
 