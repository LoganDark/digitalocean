use crate::error::RequestError;
use hyper::Response;

/// Represents a request that can be made to DigitalOcean's API. These requests
/// are executed using the `DigitalOcean` struct, which represents an API client
/// that uses a particular token.
///
/// TODO: this api feels a bit weird
#[async_trait::async_trait]
pub trait Request<T> {
	/// Performs this request with the given DigitalOcean API key.
	async fn perform(&mut self, key: &str) -> Response<T>;
}

pub type RequestResult<T> = Result<T, RequestError>;

/// Can be used to construct new `Request`s.
pub struct RequestBuilder {}

impl RequestBuilder {
	pub fn new() -> Self {
		Self {}
	}
}

impl Default for RequestBuilder {
	fn default() -> Self {
		Self {}
	}
}
