use errors::RssError;
use std::path::PathBuf;


pub trait RssConfigurable {
    fn load(path: PathBuf) -> Result<String, RssError>;
}

