use std::error::Error;
use std::convert::From;
use hyper::StatusCode;
use hyper::server::Request as HyperRequest;

use std::{fmt};


#[derive(Debug)]
/// This error is used to optionally concatenate an error with another.
/// Used as Err type in [`DefaultRssHttpServer`](struct.DefaultRssHttpServer.html)
pub struct RssError {
    /// HTTP status code
    pub status_code: StatusCode,
    /// Error description
    pub description: String,
    /// Optional parent error
    pub parent: Option<Box<Error>>,
    reason: String,
}

#[derive(Debug)]
pub struct HttpError {
    /// HTTP request
    pub request: HyperRequest,
    /// HTTP status code
    pub status_code: StatusCode,
    description: String,
}

impl HttpError {
    fn reason(status_code: StatusCode) -> String {
        match status_code.canonical_reason() {
            Some(reason) => String::from(reason),
            _ => String::from(""),
        }
    }

    pub fn new(request: HyperRequest, status_code: StatusCode) -> HttpError {
        HttpError {
            request,
            status_code,
            description: Self::reason(status_code),
        }
    }
}

impl Error for HttpError {
    fn description(&self) -> &str {
        self.description.as_str()
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} - {}",
            self.status_code.as_u16(),
            self.description.to_owned()
        )
    }
}
