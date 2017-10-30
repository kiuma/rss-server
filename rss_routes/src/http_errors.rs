use futures::prelude::*;
use hyper::{Request, Response, Error, StatusCode};
use hyper::header::ContentLength;

pub static HTML1: &'static str = "<!DOCTYPE html>
<html>
    <head>
        <meta charset=\"UTF-8\">
        <title>Page not found</title>
    </head>
    <body>
        <h1>Page not found</h1>
    </body>
</html>";
pub static HTML2: &'static str = "<!DOCTYPE html>
<html>
    <head>
        <meta charset=\"UTF-8\">
        <title>Page not found</title>
    </head>
    <body>
        <h1>Page not found</h1>
    </body>
</html>";
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
