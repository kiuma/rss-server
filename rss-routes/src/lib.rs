#![feature(proc_macro, conservative_impl_trait, generators, associated_type_defaults)]

#[macro_use]
extern crate route;

extern crate futures_await as futures;
extern crate tokio_core;
extern crate hyper_staticfile;

use hyper_staticfile::Static;
use std::path::Path;
use futures::future;
use futures::prelude::*;
use tokio_core::reactor::{Core, Handle};

extern crate hyper;

mod home;

use home::home;

mod page1;

use page1::page1;

mod http_errors;

use http_errors::HTML as HTML_ERROR;
use futures::prelude::*;

use hyper::header::ContentLength;
use hyper::Error as HyperError;
use hyper::server::{Request, Response, Service};
use hyper::StatusCode;

static PHRASE: &'static str = "Hello, World!";

type ResponseFuture = Box<Future<Item=Response, Error=HyperError>>;

pub struct RouterService {
    static_: Static,
}

impl RouterService {
    pub fn new(handle: &Handle) -> RouterService {
        RouterService {
            static_: Static::new(handle, Path::new("/tmp")),
        }
    }

    fn route(&self, req: Request) -> ResponseFuture {
        let stat_file =
            self.static_.call(req)
                .and_then(|res| {
                    let statusCode = res.status();
                    let statusRaw: u16 = statusCode.into();

                    if statusRaw >= 400 {
                        future::ok(Response::new()
                            .with_status(statusCode)
                            .with_header(ContentLength(HTML_ERROR.len() as u64))
                            .with_body(HTML_ERROR))
                    } else {
                        future::ok(res)
                    }
                });
        Box::new(stat_file)
    }
}

impl Service for RouterService {
    type Request = Request;
    type Response = Response;
    type Error = HyperError;

    type Future = ResponseFuture;

    fn call(&self, req: Request) -> Self::Future {
        let route = self.route(req);

        //        Box::new(route)
        route
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
