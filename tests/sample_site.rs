extern crate futures;
extern crate hyper;
extern crate rss_server;

use futures::future::{err, ok, Future};

use std::rc::Rc;
use hyper::server::{Request as HyperRequest, Response as HyperResponse};
use hyper::Error as HyperError;
use hyper::header::ContentLength;
use hyper::StatusCode;
use rss_server::{ErrorHandler, HttpError, Router, RouterService};

pub type ResponseFuture = Box<Future<Item = HyperResponse, Error = HyperError>>;

struct SampleRouter {
    content: String,
    path: String,
}

impl SampleRouter {
    fn new(path: &str, content: &str) -> SampleRouter {
        SampleRouter {
            content: content.to_owned(),
            path: path.to_owned(),
        }
    }
}

struct SampleErrorHandler;

impl ErrorHandler for SampleErrorHandler {
    fn dispatch(&self, error: HttpError) -> ResponseFuture {
        let status_code = error.status_code;
        let content = format!("{}", status_code.as_u16());
        let res = HyperResponse::new()
            .with_status(status_code)
            .with_header(ContentLength(content.len() as u64))
            .with_body(content);
        Box::new(ok(res))
    }
}

impl Router for SampleRouter {
    fn route(&self, req: &HyperRequest) -> Box<Future<Item = StatusCode, Error = StatusCode>> {
        if self.path == req.path() {
            Box::new(ok(StatusCode::Ok))
        } else {
            Box::new(err(StatusCode::NotFound))
        }
    }
    fn dispatch(
        &self,
        _req: HyperRequest,
        _status_code: StatusCode,
    ) -> Box<Future<Item = HyperResponse, Error = HttpError>> {
        let content = self.content.clone();
        let res = HyperResponse::new()
            .with_header(ContentLength(content.len() as u64))
            .with_body(content);
        Box::new(ok(res))
    }
}

fn get_routers() -> Vec<Rc<Router>> {
    let route1 = Rc::new(SampleRouter::new("/page1", "page1"));
    let route2 = Rc::new(SampleRouter::new("/page2", "page2"));
    let route3 = Rc::new(SampleRouter::new("/page3", "page3"));
    let mut v_routes: Vec<Rc<Router>> = Vec::new();
    v_routes.push(route1);
    v_routes.push(route2);
    v_routes.push(route3);
    v_routes
}

pub fn get_site_service() -> RouterService {
    let routes = get_routers();
    let error_handler: Rc<ErrorHandler> = Rc::new(SampleErrorHandler {});

    RouterService::new(routes, &error_handler)
}
