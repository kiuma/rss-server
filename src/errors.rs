use std::error::Error;
use std::convert::From;

use std::{io, fmt};

use toml;

#[derive(Debug)]
/// This error is used to optionally concatenate an error with another.
/// Used as Err type in [`DefaultRssHttpServer`](truct.DefaultRssHttpServer.html)
pub struct RssError {
    /// Error descrption
    pub description: String,
    /// Optional parent error
    pub parent: Option<Box<Error>>,
}

impl RssError {
    /// Creates a new RssError with the given message
    pub fn new(message: &str) -> RssError {
        RssError {
            description: String::from(message),
            parent: None,
        }
    }
}

impl Error for RssError {
    fn description(&self) -> &str {
        &self.description[..]
    }
}

impl fmt::Display for RssError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description.to_owned())
    }
}

impl From<io::Error> for RssError {
    fn from(err: io::Error) -> RssError {
        RssError {
            description: String::from(err.description()),
            parent: Some(Box::new(err)),
        }
    }
}

impl From<toml::de::Error> for RssError {
    fn from(err: toml::de::Error) -> RssError {
        RssError {
            description: String::from(err.description()),
            parent: Some(Box::new(err)),
        }
    }
}
