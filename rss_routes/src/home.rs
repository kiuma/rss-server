use futures::prelude::*;
use hyper::{Error, Response, Request};
use hyper::header::ContentLength;

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

#[async]
pub fn home(_req: Request) -> Result<Response, Error> {
    Ok(
        Response::new()
            .with_header(ContentLength(HTML.len() as u64))
            .with_body(HTML),
    )
}
