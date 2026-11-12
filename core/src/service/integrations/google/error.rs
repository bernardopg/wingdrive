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
