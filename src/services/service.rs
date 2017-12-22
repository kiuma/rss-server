use ResponseFuture;

use hyper::server::{Request as HyperRequest, Response as HyperResponse, Service as HyperService};
use std::io::Error;
use futures::future;
use futures::future::{ok, Future, FutureResult, Loop};
use hyper::StatusCode;
use hyper::Error as HyperError;
use std::rc::Rc;

pub use super::router::Router;


type RouterService = Router<
    Request = HyperRequest,
    Response = HyperResponse,
    Error = HyperError,
    Future = ResponseFuture,
>;

struct RouteResolver {
    routers: Rc<Vec<Rc<RouterService>>>,
    ix: Option<usize>,
}

impl RouteResolver {
    fn new(routers: Rc<Vec<Rc<RouterService>>>) -> RouteResolver {
        RouteResolver {
            routers: routers.clone(),
            ix: None,
        }
    }

    fn route(self, req: &HyperRequest) -> Box<Future<Item = (Self, StatusCode), Error = Error>> {
        let router = &mut self.get_router();
        match *router {
            Some(ref mut router) => Box::new(router.route(&req).then(|status_code| {
                let status_code = match status_code {
                    Ok(status_code) => status_code,
                    Err(_) => StatusCode::InternalServerError,
                };
                ok((self, status_code))
            })),
            _ => Box::new(ok((self, StatusCode::NotFound))),
        }
    }

    fn next(mut self, status_code: StatusCode) -> FutureResult<(Self, StatusCode), Error> {
        let mut current_route: usize = 0;

        match self.ix {
            Some(ix) => {
                current_route = usize::max(ix + 1, self.routers.len());
                self.ix = Some(current_route + 1);
            }
            _ => {
                if !self.routers.is_empty() {
                    self.ix = Some(current_route);
                }
            }
        }

        if status_code == StatusCode::NotFound {
            current_route + 1 >= self.routers.len();
        }

        ok((self, status_code))
    }

    fn get_router(&self) -> Option<Rc<RouterService>> {
        match self.ix {
            Some(ix) => {
                let route = self.routers.get(ix);
                match route {
                    Some(route) => Some(route.clone()),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// A RootService is a sevice that delegates the computation of an HTTP response to a list of
/// routers.
///
/// If no router is suitable for the given HTTP request, then a special RouterService, the error_handler,
/// is used to return the response. error_handler is use also to display error messages.
pub struct RootService {
    ///Vector of routers that will participate in the coice of the correct dispatcher
    routers: Rc<Vec<Rc<RouterService>>>,
    ///If no router can dispatch the response error_handler is used to display the error
    error_handler: Rc<RouterService>,
}

impl RootService {
    pub fn new(routers: Vec<Rc<RouterService>>, error_handler: Rc<RouterService>) -> RootService {
        RootService {
            routers: Rc::new(routers),
            error_handler,
        }
    }
}

impl HyperService for RootService {
    type Request = HyperRequest;
    type Response = HyperResponse;
    type Error = HyperError;
    type Future = ResponseFuture;

    fn call(&self, req: Self::Request) -> Self::Future {
        let route_resolver = RouteResolver::new(self.routers.clone());
        let e_handler = self.error_handler.clone();
        let status_code = StatusCode::NotFound;
        Box::new(
            future::loop_fn((route_resolver, req, status_code), |(route_resolver,
              req,
              _status_code)| {
                route_resolver.route(&req).and_then(
                    |(route_resolver, status_code)| {
                        route_resolver.next(status_code).and_then(|(route_resolver,
                          status_code)| {

                            let router = route_resolver.get_router();
                            match router {
                                Some(_) => {
                                    match status_code {
                                        StatusCode::NotFound => Ok(Loop::Continue(
                                            (route_resolver, req, status_code),
                                        )),
                                        _ => Ok(Loop::Break((route_resolver, req, status_code))),
                                    }
                                }
                                _ => Ok(Loop::Break((route_resolver, req, StatusCode::NotFound))),
                            }
                        })
                    },
                )
            }).then(move |route_resolver_and_req: Result<
                (RouteResolver,
                 HyperRequest,
                 StatusCode),
                Error,
            >| match route_resolver_and_req {
                Ok((route_resolver, req, status_code)) => {
                    let router = route_resolver.get_router();
                    match router {
                        Some(router) => router.dispatch(req, status_code),
                        _ => e_handler.dispatch(req, status_code),
                    }
                }
                Err(e) => panic!("This should never happen!\n{}", e),
            }),
        )
    }
}

//rss_service!(RootService, req, {
//    for route in &self.routers {
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
