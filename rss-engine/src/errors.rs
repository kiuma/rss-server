use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct RssError {
    description: String,
}

impl RssError {
    pub fn new(message: &str) -> RssError {
        RssError { description: String::from(message) }
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
