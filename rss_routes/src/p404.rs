use futures::prelude::*;
use hyper::{Request, Response, Error, StatusCode};
use hyper::header::ContentLength;

pub static HTML: &'static str = "<!DOCTYPE html>
<html>
    <head>
        <meta charset=\"UTF-8\">
        <title>Page not found</title>
    </head>
    <body>
        <h1>Page not found</h1>
    </body>
</html>";


#[async]
pub fn p404(req: Request) -> Result<Response, Error> {
    let html = format!("{}", req.path());
    Ok(Response::new()
    .with_status(StatusCode::NotFound)
        .with_header(ContentLength(HTML.len() as u64))
        .with_body(HTML))
}