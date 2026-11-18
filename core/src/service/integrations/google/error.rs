//! # Google API errors
//!
//! One error type covers OAuth, Drive and Photos so callers can tell an expired
//! grant (reconnect the account) from a transient failure (retry later) without
//! parsing messages.

use thiserror::Error;

use crate::volume::error::VolumeError;

#[derive(Error, Debug)]
pub enum GoogleError {
	/// No OAuth client is configured for this build or library.
	#[error(
		"Google integration is not configured: set an OAuth client ID in Settings > Integrations"
	)]
	NotConfigured,

	/// The user denied consent or Google rejected the authorization request.
	#[error("Google authorization failed: {0}")]
	Authorization(String),

	/// The refresh token was revoked or expired, so the account must be reconnected.
	#[error("Google account access was revoked or expired; reconnect the account")]
	InvalidGrant,

	/// The account did not grant a scope this operation needs.
	#[error(
		"The Google account did not grant access to {0}; reconnect it with that service enabled"
	)]
	MissingScope(String),

	/// The requested object does not exist.
	#[error("Not found: {0}")]
	NotFound(String),

	/// Google answered with an error status.
	#[error("Google API error ({status}): {message}")]
	Api { status: u16, message: String },

	/// The request never reached Google or the response was cut off.
	#[error("Network error talking to Google: {0}")]
	Network(String),

	/// Google answered with a body this client cannot read.
	#[error("Unexpected response from Google: {0}")]
	InvalidResponse(String),

	/// The local side of a transfer failed.
	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error),

	/// The operation cannot work on this kind of object.
	#[error("{0}")]
	Unsupported(String),

	/// The user did not finish signing in before the session expired.
	#[error("The sign-in session expired before it finished")]
	Expired,
}

impl GoogleError {
	/// Whether a retry with backoff can succeed.
	pub fn is_transient(&self) -> bool {
		match self {
			Self::Network(_) => true,
			Self::Api { status, .. } => *status == 429 || *status >= 500,
			_ => false,
		}
	}
}

impl From<reqwest::Error> for GoogleError {
	fn from(error: reqwest::Error) -> Self {
		Self::Network(error.to_string())
	}
}

impl From<serde_json::Error> for GoogleError {
	fn from(error: serde_json::Error) -> Self {
		Self::InvalidResponse(error.to_string())
	}
}

impl From<GoogleError> for VolumeError {
	fn from(error: GoogleError) -> Self {
		match error {
			GoogleError::NotFound(path) => VolumeError::Io(std::io::Error::new(
				std::io::ErrorKind::NotFound,
				format!("not found: {path}"),
			)),
			GoogleError::Io(io) => VolumeError::Io(io),
			GoogleError::Unsupported(message) => VolumeError::Unsupported(message),
			GoogleError::InvalidGrant | GoogleError::MissingScope(_) => {
				VolumeError::PermissionDenied(error.to_string())
			}
			GoogleError::Api { status: 403, .. } => {
				VolumeError::PermissionDenied(error.to_string())
			}
			other => VolumeError::Service(other.to_string()),
		}
	}
}

pub type GoogleResult<T> = Result<T, GoogleError>;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn only_network_rate_limit_and_server_errors_are_transient() {
		let api = |status| GoogleError::Api {
			status,
			message: String::new(),
		};
		assert!(GoogleError::Network("reset".into()).is_transient());
		assert!(api(429).is_transient());
		assert!(api(500).is_transient());
		assert!(api(503).is_transient());
		assert!(!api(400).is_transient());
		assert!(!api(403).is_transient());
		assert!(!api(404).is_transient());
		assert!(!GoogleError::InvalidGrant.is_transient());
		assert!(!GoogleError::Expired.is_transient());
		assert!(!GoogleError::InvalidResponse("x".into()).is_transient());
	}

	#[test]
	fn volume_errors_keep_the_meaning_of_google_errors() {
		let not_found = VolumeError::from(GoogleError::NotFound("a/b.txt".into()));
		assert!(
			matches!(&not_found, VolumeError::Io(io) if io.kind() == std::io::ErrorKind::NotFound)
		);
		assert!(matches!(
			VolumeError::from(GoogleError::Io(std::io::Error::other("disk"))),
			VolumeError::Io(_)
		));
		assert!(matches!(
			VolumeError::from(GoogleError::Unsupported("forms".into())),
			VolumeError::Unsupported(message) if message == "forms"
		));
		for denied in [
			GoogleError::InvalidGrant,
			GoogleError::MissingScope("Drive".into()),
			GoogleError::Api {
				status: 403,
				message: "forbidden".into(),
			},
		] {
			assert!(matches!(
				VolumeError::from(denied),
				VolumeError::PermissionDenied(_)
			));
		}
		assert!(matches!(
			VolumeError::from(GoogleError::Api {
				status: 500,
				message: "boom".into()
			}),
			VolumeError::Service(message) if message.contains("500") && message.contains("boom")
		));
	}

	#[test]
	fn json_errors_read_as_invalid_responses() {
		let error = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
		assert!(matches!(
			GoogleError::from(error),
			GoogleError::InvalidResponse(_)
		));
	}

	#[test]
	fn messages_tell_the_user_what_to_do() {
		assert!(GoogleError::InvalidGrant.to_string().contains("reconnect"));
		assert!(GoogleError::MissingScope("Google Photos".into())
			.to_string()
			.contains("Google Photos"));
		assert!(GoogleError::NotConfigured
			.to_string()
			.contains("Settings > Integrations"));
	}
}
