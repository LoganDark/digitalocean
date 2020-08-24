use crate::request::Request;
use crate::ratelimits::{Ratelimited, Ratelimiter};

mod error;
mod request;
mod client;
mod ratelimits;

pub mod api {
	pub mod oneclicks;
}

/// A DigitalOcean API client that can be used for making authenticated requests
/// to DigitalOcean's API. Unauthenticated endpoints are available as static
/// methods on this struct, while authenticated endpoints require you to create
/// a new instance via the `DigitalOcean::new` method.
pub struct DigitalOcean {
	/// API key
	key: String,
	/// API root (i.e. https://api.digitalocean.com/v2/)
	root: String,
	ratelimiter: Ratelimiter
}

impl DigitalOcean {
	/// Creates a new DigitalOcean API client using the provided API key. The
	/// returned client can be used for making requests to authenticated
	/// endpoints and managing account resources, but it won't perform any
	/// operations until you tell it to.
	pub fn new(key: String) -> Self {
		Self {
			key,
			root: String::from("https://api.digitalocean.com/v2/"),
			ratelimiter: Ratelimiter::new()
		}
	}

	/// Sets a new API key for this client. This will cause all new
	/// authenticated requests to use the new key. The old key will be
	/// immediately dropped, and cannot be retrieved.
	pub fn set_key(&mut self, key: String) {
		std::mem::replace(&mut self.key, key);
	}

	/// Execute a Request as this API client. By default, ratelimits will always
	/// block and you can safely unwrap the returned Result.
	pub async fn execute<T, R: Request<T>>(&mut self, req: R) -> Result<T, Ratelimited> {
		todo!()
	}
}
