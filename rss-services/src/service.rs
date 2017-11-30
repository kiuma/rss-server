use rss_engine::ResponseFuture;
//use hyper;
//use futures::future;



#[macro_export]
macro_rules! rss_service {
($struct:tt, $req:tt, $body:block) =>
(impl hyper::server::Service for $struct {
    type Request = hyper::server::Request;
    type Response = hyper::server::Response;
    type Error = hyper::Error;

    type Future = ResponseFuture;

    fn call(&self, $req: Self::Request) -> Self::Future $body
})}



//========================== TESTS =====================================================//
#[cfg(test)]
mod tests {
    use super::*;
    use hyper::server::Service;
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
        let req = hyper::Request::new(hyper::Method::Get, hyper::Uri::default());
        let resp: hyper::Response = service.call(req).wait().unwrap();

        assert_eq!(resp.headers().get(), Some(&hyper::header::ContentLength(HTML.len() as u64)))
    }
}
