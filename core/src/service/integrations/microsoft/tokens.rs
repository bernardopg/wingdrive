//! # Access tokens per Microsoft account
//!
//! Microsoft access tokens last about an hour, and every refresh also returns
//! a new refresh token. `MicrosoftTokenSource` refreshes under a lock, keeps
//! the newest refresh token in memory and hands it to `on_rotate` so the caller
//! can store it; refresh tokens that are never replaced expire after 90 days.

use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use tracing::debug;

use super::error::MicrosoftResult;
use super::oauth::{refresh_access_token, MicrosoftClient};
use super::MicrosoftEndpoints;

/// Refresh this long before the stated expiry to absorb clock skew and slow requests.
const EXPIRY_MARGIN: Duration = Duration::from_secs(120);

/// Called with each new refresh token.
pub type RotateHook = Box<dyn Fn(String) + Send + Sync>;

struct State {
	refresh_token: String,
	access_token: Option<(String, Instant)>,
}

/// Hands out valid access tokens for one account.
pub struct MicrosoftTokenSource {
	http: reqwest::Client,
	endpoints: MicrosoftEndpoints,
	client: MicrosoftClient,
	state: Mutex<State>,
	on_rotate: Option<RotateHook>,
}

impl std::fmt::Debug for MicrosoftTokenSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MicrosoftTokenSource")
			.field("client_id", &self.client.client_id)
			.finish_non_exhaustive()
	}
}

impl MicrosoftTokenSource {
	pub fn new(
		http: reqwest::Client,
		endpoints: MicrosoftEndpoints,
		client: MicrosoftClient,
		refresh_token: String,
		on_rotate: Option<RotateHook>,
	) -> Self {
		Self {
			http,
			endpoints,
			client,
			state: Mutex::new(State {
				refresh_token,
				access_token: None,
			}),
			on_rotate,
		}
	}

	/// Stores a token the caller already has, such as the one from the code exchange.
	pub async fn seed(&self, access_token: String, expires_in: u64) {
		self.state.lock().await.access_token = Some((
			access_token,
			Instant::now() + Duration::from_secs(expires_in),
		));
	}

	/// Returns a token valid for at least `EXPIRY_MARGIN`, refreshing if needed.
	pub async fn access_token(&self) -> MicrosoftResult<String> {
		let mut state = self.state.lock().await;
		if let Some((token, expires_at)) = state.access_token.as_ref() {
			if *expires_at > Instant::now() + EXPIRY_MARGIN {
				return Ok(token.clone());
			}
		}
		debug!("Refreshing Microsoft access token");
		let response = refresh_access_token(
			&self.http,
			&self.endpoints,
			&self.client,
			&state.refresh_token,
		)
		.await?;
		if let Some(rotated) = response
			.refresh_token
			.filter(|token| *token != state.refresh_token)
		{
			state.refresh_token = rotated.clone();
			if let Some(hook) = &self.on_rotate {
				hook(rotated);
			}
		}
		state.access_token = Some((
			response.access_token.clone(),
			Instant::now() + Duration::from_secs(response.expires_in),
		));
		Ok(response.access_token)
	}

	/// Drops the cached token so the next call refreshes, used after a 401.
	pub async fn invalidate(&self) {
		self.state.lock().await.access_token = None;
	}
}
