use hyper::server::Request as HyperRequest;
use futures::future::{Future, FutureResult};
use hyper::StatusCode;


///A Router is a trait meant to be used for
pub trait Router {
    /// Requests handled by the service.
    type Request;

    /// Responses given by the service.
    type Response;

    /// Errors produced by the service.
    type Error;

    /// The future response value.
    type Future: Future<Item = Self::Response, Error = Self::Error>;

    /// Process the request and return the response asynchronously.
    fn dispatch(&self, req: Self::Request, status_code: StatusCode) -> Self::Future;
    /// This method addresses the response. If the StatusCode equals to 404 (NotFound) the computation
    /// is passed to the next Router of the Resolver. If no other router can be used, the response
    //is delegated to the default error handler.
    fn route(&self, req: &HyperRequest) -> FutureResult<StatusCode, ()>;
}

#[macro_export]
macro_rules! rss_router {
($struct:tt, $req:tt, $body_route:block, $body_call:block) =>
(impl Router for $struct {
    type Request = ::hyper::server::Request;
    type Response = ::hyper::server::Response;
    type Error = ::hyper::Error;
    type Future = Box<::futures::future::Future<Item = Self::Response, Error = Self::Error>>;

    fn route(&self, $req: Self::Request) ->
        ::futures::future::FutureResult<(::hyper::StatusCode, Self::Request), ::std::io::Error>
    $body_route

    fn call(&self, $req: Self::Request) -> Self::Future
    $body_call
})}