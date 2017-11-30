use errors::RssError;

pub trait RssConfigurable {
    fn load(&self) -> Result<String, RssError>;
}

