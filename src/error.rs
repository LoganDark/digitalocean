/// Represents an error handling a particular request. This can be anything from
/// a bad status code to your account being limited.
#[derive(thiserror::Error, Debug)]
pub enum RequestError {}
