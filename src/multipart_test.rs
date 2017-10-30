use futures::{future, Future};
use hyper::header::ContentLength;
use hyper::Error as HyperError;
use hyper::server::{Request, Response, Service};

use multipart::server::RequestExt;

pub struct MultipartTest;

impl MultipartTest {
    fn write_resp(&self, x: &str) -> impl future::Future<Item=Response, Error=HyperError> {

        future::ok(Response::new()
            .with_header(ContentLength(x.len() as u64))
            .with_body(x.to_owned()))
    }
}

impl Service for MultipartTest {
    type Request = Request;
    type Response = Response;
    type Error = HyperError;

    type Future = Box<Future<Item=Response, Error=HyperError>>;


    fn call(&self, req: Request) -> Self::Future {
        let mpart_res = req
            .into_multipart();

        match mpart_res {
            Ok(_multi) => {
                Box::new(self.write_resp("multipart"))
            },
            Err(req_body) => {
                let error = format!("Not multipart {:?}", req_body);
                Box::new(self.write_resp(&error[..]))
            }

        }
    }
}