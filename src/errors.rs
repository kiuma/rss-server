use std::error::Error;
use std::convert::From;
use hyper::StatusCode;
use hyper::server::Request as HyperRequest;

use std::{io, fmt};

use toml;

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

impl RssError {
    /// Creates a new RssError with the given message
    pub fn new(status_code: StatusCode, message: Option<&str>) -> RssError {
        RssError {
            reason: Self::reason(status_code, message),
            status_code,
            description: match message {
                Some(message) => String::from(message),
                _ => Self::reason(status_code, message)
            },
            parent: None,
        }
    }
    fn reason(status_code: StatusCode, message: Option<&str>) -> String {
        let reason = match status_code.canonical_reason() {
            Some(reason) => format!(" - {}", reason),
            _ => String::from("")
        };
        match message {
            Some(message) => format!("{}{}\n{}", status_code.as_u16(),
                                     reason,
                                     message),
            _ => format!("{}{}", status_code.as_u16(),
                         reason)
        }
    }
}

impl HttpError {
    fn reason(status_code: StatusCode) -> String {
        match status_code.canonical_reason() {
            Some(reason) => String::from(reason),
            _ => String::from("")
        }
    }

    pub fn new(request: HyperRequest, status_code: StatusCode ) -> HttpError {
        HttpError{
            request,
            status_code,
            description: Self::reason(status_code),
        }
    }
}

impl Error for RssError {
    fn description(&self) -> &str {
        self.reason.as_str()
    }
}

impl Error for HttpError {
    fn description(&self) -> &str {
        self.description.as_str()
    }
}

impl fmt::Display for RssError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.status_code.as_u16(), self.description.to_owned())
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.status_code.as_u16(), self.description.to_owned())
    }
}
impl From<io::Error> for RssError {
    fn from(err: io::Error) -> RssError {
        RssError {
            reason: RssError::reason(StatusCode::InternalServerError, Some(err.description())),
            status_code: StatusCode::InternalServerError,
            description: String::from(err.description()),
            parent: Some(Box::new(err)),
        }
    }
}

impl From<toml::de::Error> for RssError {
    fn from(err: toml::de::Error) -> RssError {
        RssError {
            reason: RssError::reason(StatusCode::InternalServerError, Some(err.description())),
            status_code: StatusCode::InternalServerError,
            description: String::from(err.description()),
            parent: Some(Box::new(err)),
        }
    }
}

impl From<StatusCode> for RssError {
    fn from(err: StatusCode) -> RssError {
        RssError::new(err, None)
    }
}
