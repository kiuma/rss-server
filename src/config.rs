use errors::RssError;

///This trait is used for services that manage configuration through a load method.
pub trait RssConfigurable {
    /// This method returns a result string that the implementor uses as configuration, possibly
    /// after a deserialization.
    fn load(&self) -> Result<String, RssError>;
}
