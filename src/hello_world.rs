use futures::{future, Future};
use hyper::header::ContentLength;
use hyper::Error as HyperError;
use hyper::server::{Request, Response, Service};

pub struct HelloWorld;

impl Service for HelloWorld {
    type Request = Request;
    type Response = Response;
    type Error = HyperError;

    type Future = Box<Future<Item=Response, Error=HyperError>>;

    fn call(&self, _req: Request) -> Self::Future {
        let b =
            future::ok("hello".to_owned())
                .and_then(|x| future::ok([x, ", hello2".to_owned()].concat()))
                .and_then(|x| future::ok([x, ", hello3".to_owned()].concat()))
                .and_then(|x| future::ok([x, ", hello4".to_owned()].concat()))
                .map(|x| {
                    Response::new()
                        .with_header(ContentLength(x.len() as u64))
                        .with_body(x)
                });

        Box::new(b)
    }
}