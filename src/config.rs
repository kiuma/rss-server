use errors::RssError;

///This trait is userd for servcies that manage configuration through a load method.
pub trait RssConfigurable {
    /// This method returns a result string that the implementor may use as configuration, possibly
    /// after a deserialization.
    fn load(&self) -> Result<String, RssError>;
}
