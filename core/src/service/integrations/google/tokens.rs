//! # Access tokens per account
//!
//! Google access tokens last about an hour. `GoogleTokenSource` keeps one per
//! connected account and refreshes it under a lock, so the Drive volume and a
//! Photos upload running at the same time share a token instead of each
//! hitting the token endpoint. `GoogleApi` sends authorized requests with
//! retries on rate limits, server errors and one forced refresh on 401.

use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::{RequestBuilder, Response, StatusCode};
use tokio::sync::Mutex;
use tracing::{debug, warn};

use super::endpoints::GoogleEndpoints;
use super::error::{GoogleError, GoogleResult};
use super::oauth::{refresh_access_token, OAuthClient};

/// Refresh this long before Google's stated expiry to absorb clock skew and slow requests.
const EXPIRY_MARGIN: Duration = Duration::from_secs(120);

/// Attempts per request, counting the first.
const MAX_ATTEMPTS: u32 = 4;

#[derive(Debug)]
struct CachedToken {
	value: String,
	expires_at: Instant,
}

/// Hands out valid access tokens for one account.
#[derive(Debug)]
pub struct GoogleTokenSource {
	http: reqwest::Client,
	endpoints: GoogleEndpoints,
	client: OAuthClient,
	refresh_token: String,
	cached: Mutex<Option<CachedToken>>,
}

impl GoogleTokenSource {
	pub fn new(
		http: reqwest::Client,
		endpoints: GoogleEndpoints,
		client: OAuthClient,
		refresh_token: String,
	) -> Self {
		Self {
			http,
			endpoints,
			client,
			refresh_token,
			cached: Mutex::new(None),
		}
	}

	/// Stores a token the caller already has, such as the one from the code exchange.
	pub async fn seed(&self, access_token: String, expires_in: u64) {
		*self.cached.lock().await = Some(CachedToken {
			value: access_token,
			expires_at: Instant::now() + Duration::from_secs(expires_in),
		});
	}

	/// Returns a token valid for at least `EXPIRY_MARGIN`, refreshing if needed.
	pub async fn access_token(&self) -> GoogleResult<String> {
		let mut cached = self.cached.lock().await;
		if let Some(token) = cached.as_ref() {
			if token.expires_at > Instant::now() + EXPIRY_MARGIN {
				return Ok(token.value.clone());
			}
		}
		debug!("Refreshing Google access token");
		let response = refresh_access_token(
			&self.http,
			&self.endpoints,
			&self.client,
			&self.refresh_token,
		)
		.await?;
		let value = response.access_token.clone();
		*cached = Some(CachedToken {
			value: response.access_token,
			expires_at: Instant::now() + Duration::from_secs(response.expires_in),
		});
		Ok(value)
	}

	/// Drops the cached token so the next call refreshes, used after a 401.
	pub async fn invalidate(&self) {
		*self.cached.lock().await = None;
	}

	/// The refresh token, needed to revoke the grant.
	pub fn refresh_token(&self) -> &str {
		&self.refresh_token
	}
}

/// Authorized HTTP access to Google APIs for one account.
#[derive(Debug, Clone)]
pub struct GoogleApi {
	pub http: reqwest::Client,
	pub endpoints: GoogleEndpoints,
	pub tokens: Arc<GoogleTokenSource>,
}

impl GoogleApi {
	pub fn new(
		http: reqwest::Client,
		endpoints: GoogleEndpoints,
		tokens: Arc<GoogleTokenSource>,
	) -> Self {
		Self {
			http,
			endpoints,
			tokens,
		}
	}

	/// Sends a request built by `build`, retrying transient failures.
	///
	/// `build` runs once per attempt because a sent request cannot be reused.
	/// Rate limits (429, or 403 with a rate-limit reason) and 5xx answers back
	/// off exponentially from 500 ms. A 401 drops the cached token and retries
	/// once with a fresh one. Other error statuses come back as `GoogleError`.
	pub async fn send<F>(&self, build: F) -> GoogleResult<Response>
	where
		F: Fn(&reqwest::Client) -> RequestBuilder,
	{
		let mut refreshed = false;
		let mut attempt = 0;
		loop {
			attempt += 1;
			let token = self.tokens.access_token().await?;
			let result = build(&self.http).bearer_auth(&token).send().await;
			let response = match result {
				Ok(response) => response,
				Err(error) if attempt < MAX_ATTEMPTS => {
					warn!(%error, attempt, "Google request failed, retrying");
					tokio::time::sleep(backoff(attempt)).await;
					continue;
				}
				Err(error) => return Err(error.into()),
			};
			let status = response.status();
			if status.is_success() || status == StatusCode::PERMANENT_REDIRECT {
				return Ok(response);
			}
			if status == StatusCode::UNAUTHORIZED && !refreshed {
				refreshed = true;
				self.tokens.invalidate().await;
				continue;
			}
			let body = response.text().await.unwrap_or_default();
			let rate_limited = status == StatusCode::TOO_MANY_REQUESTS
				|| (status == StatusCode::FORBIDDEN
					&& (body.contains("rateLimitExceeded")
						|| body.contains("userRateLimitExceeded")));
			if (rate_limited || status.is_server_error()) && attempt < MAX_ATTEMPTS {
				warn!(
					status = status.as_u16(),
					attempt, "Google API throttled or failed, retrying"
				);
				tokio::time::sleep(backoff(attempt)).await;
				continue;
			}
			return Err(api_error(status, &body));
		}
	}

	/// Sends a request and parses a JSON response.
	pub async fn send_json<T, F>(&self, build: F) -> GoogleResult<T>
	where
		T: serde::de::DeserializeOwned,
		F: Fn(&reqwest::Client) -> RequestBuilder,
	{
		let response = self.send(build).await?;
		let bytes = response.bytes().await?;
		Ok(serde_json::from_slice(&bytes)?)
	}
}

/// `RequestBuilder::query` needs a reqwest feature this workspace does not
/// enable, so query strings are appended with `url`.
pub trait QueryPairs {
	fn query_pairs(self, pairs: &[(&str, &str)]) -> Self;
}

impl QueryPairs for reqwest::RequestBuilder {
	fn query_pairs(self, pairs: &[(&str, &str)]) -> Self {
		let (client, request) = self.build_split();
		match request {
			Ok(mut request) => {
				request.url_mut().query_pairs_mut().extend_pairs(pairs);
				reqwest::RequestBuilder::from_parts(client, request)
			}
			Err(error) => {
				// Keep the build error; `send` reports it.
				debug!(%error, "Could not build Google request");
				client.get("http://invalid.invalid/")
			}
		}
	}
}

fn backoff(attempt: u32) -> Duration {
	Duration::from_millis(500 * 2u64.pow(attempt.saturating_sub(1)))
}

/// Turns an error response into `GoogleError`, keeping Google's own message.
pub fn api_error(status: StatusCode, body: &str) -> GoogleError {
	let message = serde_json::from_str::<serde_json::Value>(body)
		.ok()
		.and_then(|value| {
			value
				.pointer("/error/message")
				.and_then(|m| m.as_str())
				.map(str::to_string)
		})
		.unwrap_or_else(|| body.chars().take(500).collect());
	if status == StatusCode::NOT_FOUND {
		return GoogleError::NotFound(message);
	}
	if status == StatusCode::FORBIDDEN && message.contains("insufficient") {
		return GoogleError::MissingScope(message);
	}
	GoogleError::Api {
		status: status.as_u16(),
		message,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn api_error_extracts_google_message() {
		let body = r#"{"error":{"code":404,"message":"File not found: abc."}}"#;
		assert!(matches!(
			api_error(StatusCode::NOT_FOUND, body),
			GoogleError::NotFound(m) if m == "File not found: abc."
		));
		let body =
			r#"{"error":{"code":403,"message":"Request had insufficient authentication scopes."}}"#;
		assert!(matches!(
			api_error(StatusCode::FORBIDDEN, body),
			GoogleError::MissingScope(_)
		));
		assert!(matches!(
			api_error(StatusCode::BAD_GATEWAY, "upstream"),
			GoogleError::Api { status: 502, ref message } if message == "upstream"
		));
	}

	#[test]
	fn backoff_doubles() {
		assert_eq!(backoff(1), Duration::from_millis(500));
		assert_eq!(backoff(2), Duration::from_millis(1000));
		assert_eq!(backoff(3), Duration::from_millis(2000));
	}
}
