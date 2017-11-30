use tokio_core::reactor::{Core, Handle};
use std::path::PathBuf;
//use hyper_staticfile::Static;
//
//use hyper::server::Request;
//use hyper::server::Response;
//use rss_engine::*;

pub struct StaticRouterService {
    root: PathBuf
}


impl StaticRouterService {
    pub fn new(handle: &Handle, path: PathBuf) -> StaticRouterService {
        StaticRouterService {
            root: path,
        }
    }

//    fn route(&self, req: Request) -> ResponseFuture {
//        let stat_file =
//            self.static_.call(req)
//                .and_then(|res| {
//                    let statusCode = res.status();
//                    let statusRaw: u16 = statusCode.into();
//
//                    if statusRaw >= 400 {
//                        future::ok(Response::new()
//                            .with_status(statusCode)
//                            .with_header(ContentLength(HTML_ERROR.len() as u64))
//                            .with_body(HTML_ERROR))
//                    } else {
//                        future::ok(res)
//                    }
//                });
//        Box::new(stat_file)
//    }
}