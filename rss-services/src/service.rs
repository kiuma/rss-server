use rss_engine::{ResponseFuture, RssService};
use hyper;
use hyper::server::{Request as HyperRequest, Response as HyperResponse};
use errors::HttpError;
use futures::future;
use futures::prelude::*;
use futures::future::{ok, Future, FutureResult, Loop};
use std::io::Error;
use hyper::StatusCode;

use std::rc::Rc;

#[macro_export]
macro_rules! rss_service {
($struct:tt, $req:tt, $body:block) =>
(impl HyperService for $struct {
    type Request = HyperRequest;
    type Response = HyperResponse;
    type Error = hyper::Error;

    type Future = ResponseFuture;

    fn call(&self, $req: Self::Request) -> Self::Future $body
})}

pub type RouteFuture = Box<Future<Item=Rc<RssService>, Error=HttpError>>;

pub trait Router {
    fn route(&self, req: &HyperRequest) -> FutureResult<StatusCode, Error>;
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

    fn route<'a>(&'a mut self, req: &HyperRequest) -> Box<Future<Item=(&'a mut Self, StatusCode), Error=Error> + 'a> {
        let router = self.get_router();

        match router {
            Some(router) => {
                Box::new(
                    router.route(req)
                        .map(|status_code| {
                            (self, status_code)
                        }))
            }
            _ => {
                Box::new(ok((
                    self,
                    StatusCode::NotFound,
                )))
            }
        }
    }

    fn next(&mut self, status_code: StatusCode) -> FutureResult<(&mut Self, bool), Error> {
        let mut current_route: usize = 0;
        let mut done = false;
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

        ok((
            self,
            done,
        ))
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

struct DefaultRootService {
    routes: Rc<Vec<Rc<Router>>>,
    error_handler: Rc<RssService>,
}

impl DefaultRootService {
    pub fn new(routes: Vec<Rc<Router>>, error_handler: Box<RssService>) -> DefaultRootService {
        DefaultRootService {
            routes: Rc::new(routes),
            error_handler: Rc::new(error_handler),
        }
    }

    fn dispatch(&self, req: &HyperRequest) -> ResponseFuture{
        let mut route_resolver = RouteResolver::new(self.routes.clone());
        let dispatch =
            future::loop_fn(&mut route_resolver, |mut route_resolver| {
                route_resolver.route(req)
                    .and_then(|(route_resolver, status_code)| {
                            route_resolver.next(status_code)
                                .and_then(|(route_resolver, done)| {
                                    let router = route_resolver.get_router();
                                    match router {
                                        Some(router) => {
                                            if done {
                                                Ok(Loop::Break(route_resolver))
                                            } else {
                                                Ok(Loop::Continue(route_resolver))
                                            }
                                        },
                                        _ => Ok(Loop::Break(route_resolver)),
                                    }
                                })
                    })
            });
        let dispatch_result =
            dispatch.and_then(|route_resolver | {
            let router = route_resolver.get_router();
            match router {
//                Some(router) => {
                Some(_) => {
//                    router.route(req)
                    let e_handler = self.error_handler.clone();
                    e_handler.call(*req);
                },
                _ => {
                    let e_handler = self.error_handler.clone();
                    e_handler.call(*req);
                },
            }
        });
        Box::new(dispatch_result)
    }
}

impl hyper::server::Service for DefaultRootService {
    type Request = HyperRequest;
    type Response = HyperResponse;
    type Error = hyper::Error;
    type Future = ResponseFuture;


    fn call(&self, req: Self::Request) -> Self::Future {
        //let route = &self.select_route(&req);
        //        for route in &self.routes {
        //            let service_result = await!(route.route(&req));
        ////            dispatch item on Ok...., continue looping on Error
        //        }
        Box::new(future::ok(HyperResponse::new()))
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
    use super::*;
    use HyperService;
    use futures::future;
    use futures::prelude::*;

    static HTML: &'static str = "<!DOCTYPE html>
<html>
    <head>
        <meta charset=\"UTF-8\">
        <title>Home</title>
    </head>
    <body>
        <h1>This is home!</h1>
    </body>
</html>";

    #[test]
    fn test_rss_service_macro_def() {
        pub struct FooService;
        rss_service!(FooService, _, {
    Box::new(future::ok(
    Self::Response::new()
    .with_header(hyper::header::ContentLength(HTML.len() as u64))
    .with_body(HTML),
    ))
    });
        let service = FooService {};
        let req = HyperRequest::new(hyper::Method::Get, hyper::Uri::default());
        let resp: hyper::Response = service.call(req).wait().unwrap();

        assert_eq!(
            resp.headers().get(),
            Some(&hyper::header::ContentLength(HTML.len() as u64))
        )
    }
}
