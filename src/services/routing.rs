use HttpError;
use hyper::server::{Request as HyperRequest, Response as HyperResponse, Service as HyperService};
use std::io::Error;
use futures::future;
use futures::future::{err, ok, Future, Loop};
use hyper::StatusCode;
use hyper::Error as HyperError;
use std::rc::Rc;

pub type ResponseFuture = Box<Future<Item = HyperResponse, Error = HyperError>>;

pub type RssService = HyperService<
    Request = HyperRequest,
    Response = HyperResponse,
    Error = HyperError,
    Future = ResponseFuture,
>;

/// A `Router` is a trait meant to be used for addressing requests.
///
/// Routers are usually chained together in a vector and passed to a [`RouterService`](struct.RouterService.html)
/// When a request arrives, the asynchronous [`route`](trait.Router.html#tymethod.route) method is called.
///
/// - If the future result is NOT an error, the response generation is delegated to the [`dispatch`](trait.Router.html#tymethod.dispatch) method.
/// - If the future result is an error, and the associated status code is `NotFound` (404) the computation is delegated to the
/// next `Router` of [`RouterService`](struct.RouterService.html). If every route has been accepted and has failed or if the status code associated
/// to the error is different from `NotFound`. The special Router `error_handler` of [`RouterService`](struct.RouterService.html) is used to display
/// the error message.
///
/// Routers are usually passed to [`RouterService::new`](struct.RouterService.html#tymethod.new), an [Hyper](https://hyper.rs/)
/// that performs HTTP dispatching strategy
pub trait Router {
    /// This method is used to perform the routing logic. If the status code is not returned as an error,
    /// the [`dispatch`](trait.Router.html#tymethod.dispatch) method will be called to render the HTTP
    /// response.
    fn route(&self, req: &HyperRequest) -> Box<Future<Item = StatusCode, Error = StatusCode>>;

    /// This method processes the request and return the response asynchronously. If the future resolves to an error,
    /// the response generation is delegated to the [`RouterService`](struct.RouterService.html) `error_handler`.
    fn dispatch(
        &self,
        req: HyperRequest,
        status_code: StatusCode,
    ) -> Box<Future<Item = HyperResponse, Error = HttpError>>;
}

pub trait ErrorHandler: Sync + Send {
    /// This method processes the request and return the response asynchronously..
    fn dispatch(&self, http_error: HttpError) -> ResponseFuture;
}

struct RouteResolver {
    routers: Rc<Vec<Rc<Router>>>,
    ix: usize,
}

impl RouteResolver {
    fn new(routers: &Rc<Vec<Rc<Router>>>) -> RouteResolver {
        RouteResolver {
            routers: Rc::clone(routers),
            ix: 0,
        }
    }

    fn route(
        self,
        req: &HyperRequest,
    ) -> Box<Future<Item = (Self, StatusCode), Error = (Self, StatusCode)>> {
        let router = &mut self.get_router();
        match *router {
            Some(ref mut router) => {
                Box::new(router.route(req).then(|status_code| match status_code {
                    Ok(status_code) => ok((self, status_code)),
                    Err(status_code) => err((self, status_code)),
                }))
            }
            _ => Box::new(err((self, StatusCode::NotFound))),
        }
    }

    fn next(mut self) -> Result<Self, Self> {
        if self.ix + 1 >= self.routers.len() {
            self.ix = self.routers.len();
            Err(self)
        } else {
            self.ix += 1;
            Ok(self)
        }
    }

    fn get_router(&self) -> Option<Rc<Router>> {
        let route = self.routers.get(self.ix);
        match route {
            Some(route) => Some(Rc::clone(route)),
            _ => None,
        }
    }
}

/// A `RouterService` is a service that delegates the computation of an HTTP response to a list of
/// routers (see [`Router`](trait.Router.html)).
///
/// If no router is suitable for the given HTTP request, then a special `Router`, the `error_handler`,
/// is used to return the response. `error_handler` is used to render error messages.
pub struct RouterService {
    ///Vector of routers that will participate in the choice of the correct dispatcher
    routers: Rc<Vec<Rc<Router>>>,
    ///If no router can dispatch the response, error_handler is used to render the error
    error_handler: Rc<ErrorHandler>,
}

impl RouterService {
    /// Creates a new root service
    ///
    /// - `routers`: Vector of routers that will participate in the choice of the correct dispatcher
    /// - `error_handler`: This special `Router` is invoked when no routes can resolve the request or when a
    /// `Router` returns an error different from a `NotFound` (404).
    pub fn new(routers: Vec<Rc<Router>>, error_handler: &Rc<ErrorHandler>) -> RouterService {
        RouterService {
            routers: Rc::new(routers),
            error_handler: Rc::clone(error_handler),
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
        let e_handler = Rc::clone(&self.error_handler);
        let status_code = StatusCode::NotFound;
        Box::new(
            future::loop_fn(
                (route_resolver, req, status_code),
                |(route_resolver, req, _status_code)| {
                    route_resolver
                        .route(&req)
                        .then(|route_result| match route_result {
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
                            Err((route_resolver, status_code)) => match status_code {
                                StatusCode::NotFound => match route_resolver.next() {
                                    Ok(route_resolver) => Ok(Loop::Continue((
                                        route_resolver,
                                        req,
                                        StatusCode::NotFound,
                                    ))),
                                    Err(route_resolver) => {
                                        Ok(Loop::Break((route_resolver, req, StatusCode::NotFound)))
                                    }
                                },
                                _ => Ok(Loop::Break((route_resolver, req, status_code))),
                            },
                        })
                },
            ).then(
                |route_resolver_and_req: Result<
                    (RouteResolver, HyperRequest, StatusCode),
                    Error,
                >| match route_resolver_and_req {
                    Ok((route_resolver, req, status_code)) => {
                        let router = route_resolver.get_router();
                        match router {
            Some(router) => router.dispatch(req, status_code),
            _ => Box::new(err(HttpError::new(req, StatusCode::NotFound)))//e_handler.dispatch(req, status_code),
        }
                    }
                    Err(e) => panic!("This should never happen!\n{}", e),
                },
            )
                .then(move |dispatch_result| match dispatch_result {
                    Ok(res) => Box::new(ok(res)),
                    Err(http_error) => e_handler.dispatch(http_error),
                }),
        )
    }
}

//========================== TESTS =====================================================//
#[cfg(test)]
mod tests {
    // extern crate http;

    use super::*;
    use hyper::header::ContentLength;
    use hyper::Method;
    use futures::Stream;
    use std::str::from_utf8;

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
            if &self.path == req.path() {
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

    fn dispatch_to_string(response: HyperResponse) -> String {
        let body = response.body();
        let body_content = body.concat2()
            .and_then(|body| {
                let stringify = String::from(from_utf8(&body).unwrap());
                ok(stringify)
            })
            .wait()
            .unwrap();
        body_content
    }

    fn test_resolver_to_page(page: &str, router_resolver: RouteResolver) {
        let uri = format!("https://www.rss-server.org/{}", page)
            .parse()
            .unwrap();
        let req = HyperRequest::new(Method::Get, uri);

        let route_result = router_resolver.route(&req).wait().ok();

        let (router_resolver, status_code) = match route_result {
            Some((router_resolver, status_code)) => (router_resolver, status_code),
            None => panic!("No route found for {}", req.path()),
        };
        //let (router_resolver, status_code) = router_resolver.route(&req).wait().ok().unwrap();
        assert_eq!(status_code, StatusCode::Ok);

        let router = router_resolver.get_router().unwrap();
        let response = router.dispatch(req, status_code).wait().unwrap();

        let body = dispatch_to_string(response);

        assert_eq!(body, page, "Expetted: \"{}\", got \"{}\"", page, body);
    }

    #[test]
    fn test_resolver_to_page1() {
        let routes = Rc::new(get_routers());
        let router_resolver = RouteResolver::new(&routes);

        test_resolver_to_page("page1", router_resolver);
    }

    #[test]
    fn test_resolver_to_page2() {
        let routes = Rc::new(get_routers());
        let mut router_resolver = RouteResolver::new(&routes);
        router_resolver = router_resolver.next().ok().unwrap();

        test_resolver_to_page("page2", router_resolver);
    }

    #[test]
    fn test_resolver_to_page3() {
        let routes = Rc::new(get_routers());
        let mut router_resolver = RouteResolver::new(&routes);
        router_resolver = router_resolver.next().ok().unwrap();
        router_resolver = router_resolver.next().ok().unwrap();

        test_resolver_to_page("page3", router_resolver);
    }

    #[test]
    fn test_resolver_max_resolvers() {
        let routes = Rc::new(get_routers());
        let n_resolvers = routes.len();
        let mut router_resolver = RouteResolver::new(&routes);

        router_resolver.ix = n_resolvers - 1;

        assert!(router_resolver.get_router().is_some());
        let router_resolver = router_resolver.next();
        assert!(router_resolver.is_err());
    }

    fn test_router_service_to_page(page: &str) {
        let uri = format!("https://www.rss-server.org/{}", page)
            .parse()
            .unwrap();
        let req = HyperRequest::new(Method::Get, uri);

        let routes = get_routers();
        let error_handler: Rc<ErrorHandler> = Rc::new(SampleErrorHandler {});

        let router_service = RouterService::new(routes, &error_handler);

        let response = router_service.call(req).wait().ok().unwrap();

        assert_eq!(response.status(), StatusCode::Ok);

        let body = dispatch_to_string(response);

        assert_eq!(body, page, "Expetted: \"{}\", got \"{}\"", page, body);
    }

    #[test]
    fn test_router_service_to_page1() {
        test_router_service_to_page("page1");
    }

    #[test]
    fn test_router_service_to_page2() {
        test_router_service_to_page("page2");
    }

    #[test]
    fn test_router_service_to_page3() {
        test_router_service_to_page("page3");
    }

    #[test]
    fn test_router_service_to_page_not_found() {
        let uri = "https://www.rss-server.org/notFound".parse().unwrap();
        let req = HyperRequest::new(Method::Get, uri);

        let routes = get_routers();
        let error_handler: Rc<ErrorHandler> = Rc::new(SampleErrorHandler {});

        let router_service = RouterService::new(routes, &error_handler);

        let response = router_service.call(req).wait().ok().unwrap();

        assert_eq!(response.status(), StatusCode::NotFound);

        let body = dispatch_to_string(response);
        let expected = format!("{}", (StatusCode::NotFound).as_u16());
        assert_eq!(
            body,
            expected,
            "Expetted: \"{}\", got \"{}\"",
            expected,
            body
        );
    }
}
