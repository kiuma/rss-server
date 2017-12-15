use std::error::Error;
use std::convert::From;

use std::{io, fmt};

#[derive(Debug)]
pub struct HttpError {
    code: u16,
    description: String,
    parent: Option<Box<Error>>
}

impl HttpError {
    pub fn new(code: u16, message: &str) -> HttpError {
        HttpError { code, description: String::from(message), parent: None }
    }
}

impl Error for HttpError {
    fn description(&self) -> &str {
        &self.description[..]
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description.to_owned())
    }
}

impl From<io::Error> for HttpError {
    fn from(err: io::Error) -> HttpError {
        HttpError {
            code: 500,
            description : String::from(err.description()),
            parent: Some(Box::new(err))
        }
    }
}
