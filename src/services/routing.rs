use ResponseFuture;

use hyper::server::{Request as HyperRequest, Response as HyperResponse, Service as HyperService};
use std::io::Error;
use futures::future;
use futures::future::{ok, err, Future, Loop};
use hyper::StatusCode;
use hyper::Error as HyperError;

use std::sync::Arc;


/// A `Router` is a trait meant to be used for addressing requests.
///
/// Routers are usualy chained together in a vector and passed to a [`RouterService`](struct.RouterService.html)
/// When a request arrives, the asynchronous [`route`](trait.Router.html#tymethod.route) method is called.
///
/// - If the future result is NOT an error, the response generation is delegated to the [`dispatch`](trait.Router.html#tymethod.dispatch) method.
/// - If the future result is an error, and the associated status code is `NotFound` (404) the computation is delegated to the
/// next `Router` of [`RouterService`](struct.RouterService.html). If every route has been attepted and has failed or if the status code associated
/// to the error is different from `NotFound`. The special Router `error_handler` of [`RouterService`](struct.RouterService.html) is usedto display
/// the error message.
///
/// Routers are usually passed to [`RouterService::new`](struct.RouterService.html#tymethod.new), an [Hyper](https://hyper.rs/)
/// that performs HTTP dispatching strategy
pub trait Router: Sync + Send {
    /// This method is used to perform the routing logic. If the status code is not returned as an error,
    /// the [`dispatch`](trait.Router.html#tymethod.dispatch) method will be called to render the HTTP
    /// response.
    fn route(&self, req: &HyperRequest) -> Box<Future<Item = StatusCode, Error = StatusCode>>;

    /// This method processes the request and return the response asynchronously. If the future resolves to an error,
    /// the response generation is delegeted to the [`RouterService`](struct.RouterService.html) `error_handler`.
    fn dispatch(
        &self,
        req: HyperRequest,
        status_code: StatusCode,
    ) -> Box<Future<Item = HyperResponse, Error = HyperError>>;
}

struct RouteResolver {
    routers: Arc<Vec<Arc<Router>>>,
    ix: usize,
}

impl RouteResolver {
    fn new(routers: &Arc<Vec<Arc<Router>>>) -> RouteResolver {
        RouteResolver {
            routers: Arc::clone(routers),
            ix: 0,
        }
    }

    fn route(
        self,
        req: &HyperRequest,
    ) -> Box<Future<Item = (Self, StatusCode), Error = (Self, StatusCode)>> {
        let router = &mut self.get_router();
        match *router {
            Some(ref mut router) => Box::new(
                router.route(req).then(|status_code| match status_code {
                    Ok(status_code) => ok((self, status_code)),
                    Err(status_code) => err((self, status_code)),
                }),
            ),
            _ => Box::new(err((self, StatusCode::NotFound))),
        }
    }

    fn next(mut self) -> Result<Self, Self> {
        if self.ix + 1 >= self.routers.len() {
            Err(self)
        } else {
            self.ix += 1;
            Ok(self)
        }
    }

    fn get_router(&self) -> Option<Arc<Router>> {
        let route = self.routers.get(self.ix);
        match route {
            Some(route) => Some(Arc::clone(route)),
            _ => None,
        }
    }
}

/// A `RouterService` is a sevice that delegates the computation of an HTTP response to a list of
/// routers (see [`Router`](trait.Router.html)).
///
/// If no router is suitable for the given HTTP request, then a special `Router`, the `error_handler`,
/// is used to return the response. `error_handler` is used to render error messages.
pub struct RouterService {
    ///Vector of routers that will participate in the coice of the correct dispatcher
    routers: Arc<Vec<Arc<Router>>>,
    ///If no router can dispatch the response, error_handler is used to render the error
    error_handler: Arc<Router>,
}

impl RouterService {
    /// Creates a new root service
    ///
    /// - `routers`: Vector of routers that will participate in the choice of the correct dispatcher
    /// - `error_handler`: This special `Router` is invoked when no routes can resolve the request or when a
    /// `Router` returns an error different from a `NotFound` (404).
    pub fn new(routers: Vec<Arc<Router>>, error_handler: &Arc<Router>) -> RouterService {
        RouterService {
            routers: Arc::new(routers),
            error_handler: Arc::clone(error_handler),
        }
    }
}

impl HyperService for RouterService {
    type Request = HyperRequest;
    type Response = HyperResponse;
    type Error = HyperError;
    type Future = ResponseFuture;

    fn call(&self, req: Self::Request) -> Self::Future {
        let route_resolver = RouteResolver::new(&self.routers);
        let e_handler = Arc::clone(&self.error_handler);
        let status_code = StatusCode::NotFound;
        Box::new(
            future::loop_fn((route_resolver, req, status_code), |(route_resolver,
              req,
              _status_code)| {
                route_resolver.route(&req).then(|route_result| {

                    match route_result {
                        Ok((route_resolver, status_code)) => {
                            let router = route_resolver.get_router();
                            Ok(Loop::Break((
                                route_resolver,
                                req,
                                match router {
                                    Some(_) => status_code,
                                    _ => StatusCode::NotFound,
                                },
                            )))
                        }
                        Err((route_resolver, status_code)) => {
                            match status_code {
                                StatusCode::NotFound => {
                                    match route_resolver.next() {
                                        Ok(route_resolver) => Ok(Loop::Continue((
                                            route_resolver,
                                            req,
                                            StatusCode::NotFound,
                                        ))),
                                        Err(route_resolver) => Ok(Loop::Break((
                                            route_resolver,
                                            req,
                                            StatusCode::NotFound,
                                        ))),
                                    }
                                }
                                _ => Ok(Loop::Break((route_resolver, req, status_code))),
                            }
                        }
                    }
                })
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
