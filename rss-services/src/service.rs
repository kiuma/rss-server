use rss_engine::{ResponseFuture, RssService};
use hyper;
use hyper::server::{Request as HyperRequest, Response as HyperResponse};
use futures::future;
// use futures::prelude::*;
use futures::future::{ok, Future, FutureResult, Loop};
use std::io::Error;
use hyper::StatusCode;
use hyper::Error as HyperError;

use std::rc::Rc;

#[macro_export]
macro_rules! rss_service {
($struct:tt, $req:tt, $body:block) =>
(impl hyper::server::Service for $struct {
    type Request = HyperRequest;
    type Response = HyperResponse;
    type Error = hyper::Error;

    type Future = ResponseFuture;

    fn call(&self, $req: Self::Request) -> Self::Future $body
})}


pub trait Router {
    fn route(&self, req: HyperRequest) -> FutureResult<(StatusCode, HyperRequest), Error>;
}

struct RouteResolver {
    routes: Rc<Vec<Rc<Router>>>,
    ix: Option<usize>,
}

impl RouteResolver {
    fn new(routes: Rc<Vec<Rc<Router>>>) -> RouteResolver {
        RouteResolver {
            routes: routes.clone(),
            ix: None,
        }
    }

    fn route(
        self,
        req: HyperRequest,
    ) -> Box<Future<Item = (Self, StatusCode, HyperRequest), Error = Error>> {
        let router = &mut self.get_router();
        match *router {
            Some(ref mut router) => Box::new(router.route(req).map(|(status_code, req)| {
                (self, status_code, req)
            })),
            _ => Box::new(ok((self, StatusCode::NotFound, req))),
        }
    }

    fn next(mut self, status_code: StatusCode) -> FutureResult<(Self, bool), Error> {
        let mut current_route: usize = 0;

        match self.ix {
            Some(ix) => {
                current_route = usize::max(ix + 1, self.routes.len());
                self.ix = Some(current_route + 1);
            }
            _ => {
                if !self.routes.is_empty() {
                    self.ix = Some(current_route);
                }
            }
        }
        let done = match status_code {
            StatusCode::NotFound => current_route + 1 >= self.routes.len(),
            _ => true,
        };
        ok((self, done))
    }

    fn get_router(&self) -> Option<Rc<Router>> {
        match self.ix {
            Some(ix) => {
                let route = self.routes.get(ix);
                match route {
                    Some(route) => Some(route.clone()),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

pub struct DefaultRootService {
    routes: Rc<Vec<Rc<Router + 'static>>>,
    error_handler: Rc<RssService>,
}

impl DefaultRootService {
    pub fn new(routes: Vec<Rc<Router>>, error_handler: Box<RssService>) -> DefaultRootService {
        DefaultRootService {
            routes: Rc::new(routes),
            error_handler: Rc::new(error_handler),
        }
    }
}

impl hyper::server::Service for DefaultRootService {
    type Request = HyperRequest;
    type Response = HyperResponse;
    type Error = HyperError;
    type Future = ResponseFuture;

    fn call(&self, req: Self::Request) -> Self::Future {
        let route_resolver = RouteResolver::new(self.routes.clone());
        let e_handler = self.error_handler.clone();
        Box::new(
            future::loop_fn((route_resolver, req), |(route_resolver, req)| {

                route_resolver.route(req).and_then(|(route_resolver,
                  status_code,
                  req)| {
                    route_resolver.next(status_code).and_then(
                        |(route_resolver, done)| {

                            let router = route_resolver.get_router();
                            match router {
                                Some(_) => {
                                    if done {
                                        Ok(Loop::Break((route_resolver, req)))
                                    } else {
                                        Ok(Loop::Continue((route_resolver, req)))
                                    }
                                }
                                _ => Ok(Loop::Break((route_resolver, req))),
                            }
                        },
                    )
                })
            }).then(move |route_resolver_and_req| {
                match route_resolver_and_req {
                    Ok((route_resolver, req)) => {
                        let router = route_resolver.get_router();
                        match router {
                            //                Some(router) => {
                            Some(_) => e_handler.call(req),
                            _ => e_handler.call(req),
                        }
                    }
                    Err(_) => panic!("This sholuld never happen"),
                }
            }),
        )
    }
}

//rss_service!(DefaultRootService, req, {
//    for route in &self.routes {
//        println!("{}", route);
//    }
//    Box::new(future::ok(Self::Response::new()))
//});

//========================== TESTS =====================================================//
#[cfg(test)]
mod tests {
    //     use super::*;
    //     use futures::future;
    //     use futures::prelude::*;
    //
    //     static HTML: &'static str = "<!DOCTYPE html>
    // <html>
    //     <head>
    //         <meta charset=\"UTF-8\">
    //         <title>Home</title>
    //     </head>
    //     <body>
    //         <h1>This is home!</h1>
    //     </body>
    // </html>";
    //
    //     #[test]
    //     fn test_rss_service_macro_def() {
    //         pub struct FooService;
    //         rss_service!(FooService, _, {
    //     Box::new(future::ok(
    //     Self::Response::new()
    //     .with_header(hyper::header::ContentLength(HTML.len() as u64))
    //     .with_body(HTML),
    //     ))
    //     });
    //         let service = FooService {};
    //         let req = HyperRequest::new(hyper::Method::Get, hyper::Uri::default());
    //         let resp: hyper::Response = service.call(req).wait().unwrap();
    //
    //         assert_eq!(
    //             resp.headers().get(),
    //             Some(&hyper::header::ContentLength(HTML.len() as u64))
    //         )
    //     }
}
