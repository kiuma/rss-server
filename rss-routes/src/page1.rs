use futures::prelude::*;
use hyper::{Request, Response, Error};
use hyper::header::ContentLength;

static HTML: &'static str = "<!DOCTYPE html>
<html>
    <head>
        <meta charset=\"UTF-8\">
        <title>Page 1</title>
    </head>
    <body>
        <h1>This is page 1!</h1>
    </body>
</html>";


#[async]
pub fn page1(_req: Request) -> Result<Response, Error> {
    Ok(Response::new()
        .with_header(ContentLength(HTML.len() as u64))
        .with_body(HTML))
}