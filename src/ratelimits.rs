use crate::request::Request;
use std::time::{Instant, UNIX_EPOCH, SystemTime};
use hyper::Response;
use hyper::http::response::Parts;
use tokio::time::Duration;
use log::info;
use std::future::Future;
use std::pin::Pin;

/// You get this struct if you are currently ratelimited, but don't want the
/// Ratelimiter to wait on it. It's probably a rare case but the option is there
/// if you need it. Usually, you won't ever see this struct, since it's handled
/// internally by the `Ratelimiter` by default, but if you modify the ratelimit
/// policy to `RespectNonblocking`, you will get `Err`s containing these if you
/// ever hit a ratelimit.
#[derive(Debug)]
pub struct Ratelimited {
	until: SystemTime,
	cached: bool
}

impl Ratelimited {
	fn new(until: SystemTime, cached: bool) -> Self {
		Self { until, cached }
	}

	/// Returns the time at which this ratelimit expires. Note that a request is
	/// not guaranteed to succeed after this time, because multiple clients may
	/// be competing to send a request, or the system clock may be off.
	pub fn until(&self) -> &SystemTime {
		&self.until
	}

	/// Returns true if this was a cached result, i.e. no request was actually
	/// sent to DigitalOcean's servers yet, but we know that one would have
	/// been rejected because we paid attention to the previous response's
	/// headers.
	pub fn cached(&self) -> bool {
		self.cached
	}

	/// Waits until this ratelimit is up. Note that if your system clock is
	/// ahead, this may finish before DigitalOcean has *actually* forgotten the
	/// oldest request, so make sure your clock is accurate I guess...
	pub async fn wait(self) {
		let until = self.until.duration_since(SystemTime::now())
			.unwrap_or_else(Duration::default);
		let instant = tokio::time::Instant::from_std(Instant::now() + until);

		tokio::time::delay_until(instant).await;
	}
}

/// Specifies the ratelimiter's policy on ratelimits.
#[derive(Debug)]
pub enum RatelimitPolicy {
	/// The default. When trying to execute a request, wait until we can send
	/// the request. If we think we can, but we get rejected, wait and try
	/// again.
	RespectBlocking,

	/// When trying to execute a request, if we are currently ratelimited,
	/// immediately return with an `Err(Ratelimited)`. This is the only policy
	/// where a Ratelimited will ever be returned.
	RespectNonblocking,

	/// Ignore all ratelimits. You may receive a 429 status code, in which case
	/// the request will **not** be retried.
	Ignore
}

/// The Ratelimiter is used internally by the DigitalOcean struct to handle API
/// ratelimits.
///
/// DigitalOcean uses a sliding window system for their ratelimits, which makes
/// the implementation a little involved but also more flexible than a fixed "X
/// per hour" ratelimit system.
///
/// The Ratelimiter handles these ratelimits by continually updating itself with
/// each completed request and estimating when we'll be ratelimited and when we
/// can continue sending requests. If we hit a ratelimit it will intelligently
/// re-send the request once the ratelimit is up.
///
/// # Concurrency
/// Ratelimiters are not concurrent and cannot execute multiple requests in
/// parallel. You should use a mutex when accessing it, or any other mechanism
/// that can avoid calling `execute` more than once at a time.
#[derive(Debug)]
pub struct Ratelimiter {
	policy: RatelimitPolicy,

	ratelimit_limit: u16,
	ratelimit_reset: Option<SystemTime>,
	ratelimit_remaining: u16
}

impl Ratelimiter {
	/// Creates a new Ratelimiter.
	pub fn new() -> Self {
		Self {
			policy: RatelimitPolicy::RespectBlocking,

			// Give ourselves no limit to start. As soon as we send our first
			// request, we'll know what the situation is.
			ratelimit_limit: u16::max_value(),
			ratelimit_reset: None,
			ratelimit_remaining: u16::max_value(),
		}
	}

	/// If the ratelimit should have expired by now, reset the cache
	fn cache_reset_if_needed(&mut self) {
		if self.ratelimit_reset.is_some() && SystemTime::now() > self.ratelimit_reset.unwrap() {
			info!("Resetting ratelimiter; current time is after {:?}", self.ratelimit_reset);

			// Since DigitalOcean uses a rolling window, not all requests are
			// going to expire at once. Even though this is reset on every
			// new request, and Ratelimiter is not concurrent right now, it
			// still makes sense to only increase this by 1 for our purposes,
			// since DigitalOcean only guarantees that exactly one request will
			// expire at the provided timestamp.
			self.ratelimit_remaining += 1;
			self.ratelimit_reset = None;
		}
	}

	/// Returns a Ratelimited if we know that the API server would ratelimit us
	fn cache_ratelimited(&self) -> Option<Ratelimited> {
		if let Some(reset) = self.ratelimit_reset {
			if self.ratelimit_remaining == 0 {
				Some(Ratelimited::new(reset, true))
			} else {
				None
			}
		} else {
			// `ratelimit_reset` is None. We are either newly created, or a
			// previous ratelimit has expired. As far as the cache is concerned,
			// we're all set.
			None
		}
	}

	/// Studies the headers of a request head, picking out the three ratelimit
	/// headers and applying them to our internal data
	fn study_headers(&mut self, head: &Parts) -> Result<(), &str> {
		let ratelimit_limit: u16 = head.headers.get("RateLimit-Limit")
			.ok_or("no RateLimit-Limit header")?.to_str()?.parse()?;
		let ratelimit_remaining: u16 = head.headers.get("RateLimit-Remaining")
			.ok_or("no RateLimit-Remaining header")?.to_str()?.parse()?;
		let ratelimit_reset: u64 = head.headers.get("RateLimit-Reset")
			.ok_or("no RateLimit-Reset header")?.to_str()?.parse()?;

		let ratelimit_reset = UNIX_EPOCH + Duration::from_secs(ratelimit_reset);

		self.ratelimit_limit = ratelimit_limit;
		self.ratelimit_reset = Some(ratelimit_reset);
		self.ratelimit_remaining = ratelimit_remaining;

		Ok(())
	}

	/// Executes the specified request with the specified API key.
	///
	/// # Panics
	/// Panics if a response from DigitalOcean does not contain correct
	/// ratelimit headers. This should, honestly, never happen...
	pub fn execute<'a, T, R: Request<T> + 'a>(&'a mut self, mut req: R, key: &'a str)
		-> Pin<Box<dyn Future<Output = Result<Response<T>, Ratelimited>> + 'a>> {
		Box::pin(async move {
			self.cache_reset_if_needed();

			if let Some(ratelimited) = self.cache_ratelimited() {
				info!("Pretty sure we will be ratelimited, {:?}", ratelimited);

				match self.policy {
					RatelimitPolicy::RespectBlocking => {
						ratelimited.wait().await;
					},
					RatelimitPolicy::RespectNonblocking => {
						return Err(ratelimited);
					},
					RatelimitPolicy::Ignore => {}
				}
			}

			let response = req.perform(key).await;
			let (head, body) = response.into_parts();

			let result = self.study_headers(&head);

			if result.is_err() {
				panic!("Couldn't study ratelimit headers: {}", result.unwrap_err());
			}

			if head.status == 429 {
				match self.policy {
					RatelimitPolicy::RespectBlocking => {
						return self.execute(req, key).await;
					},
					RatelimitPolicy::RespectNonblocking => {
						return Err(Ratelimited::new(self.ratelimit_reset.unwrap(), false))
					},
					RatelimitPolicy::Ignore => {}
				}
			}

			Ok(Response::from_parts(head, body))
		})
	}
}
