//! # Microsoft errors
//!
//! One error type for sign-in, token refresh and Graph calls, so callers can
//! tell an expired grant (reconnect the account) from a transient failure.

use thiserror::Error;

use crate::service::integrations::google::GoogleError;
use crate::volume::error::VolumeError;

#[derive(Error, Debug)]
pub enum MicrosoftError {
	/// The user denied consent or Microsoft rejected the request.
	#[error("Microsoft authorization failed: {0}")]
	Authorization(String),

	/// The refresh token was revoked or expired, so the account must be reconnected.
	#[error("Microsoft account access was revoked or expired; reconnect the account")]
	InvalidGrant,

	/// Microsoft answered with an error status.
	#[error("Microsoft API error ({status}): {message}")]
	Api { status: u16, message: String },

	/// The request never reached Microsoft or the response was cut off.
	#[error("Network error talking to Microsoft: {0}")]
	Network(String),

	/// Microsoft answered with a body this client cannot read.
	#[error("Unexpected response from Microsoft: {0}")]
	InvalidResponse(String),

	/// The user did not finish signing in before the session expired.
	#[error("The sign-in session expired before it finished")]
	Expired,
}

impl From<reqwest::Error> for MicrosoftError {
	fn from(error: reqwest::Error) -> Self {
		Self::Network(error.to_string())
	}
}

impl From<serde_json::Error> for MicrosoftError {
	fn from(error: serde_json::Error) -> Self {
		Self::InvalidResponse(error.to_string())
	}
}

impl From<std::io::Error> for MicrosoftError {
	fn from(error: std::io::Error) -> Self {
		Self::Network(error.to_string())
	}
}

/// The loopback listener is shared with Google and reports its errors as `GoogleError`.
impl From<GoogleError> for MicrosoftError {
	fn from(error: GoogleError) -> Self {
		match error {
			GoogleError::Expired => Self::Expired,
			GoogleError::Authorization(message) => Self::Authorization(message),
			GoogleError::Io(io) => Self::Network(io.to_string()),
			other => Self::Authorization(other.to_string()),
		}
	}
}

impl From<MicrosoftError> for VolumeError {
	fn from(error: MicrosoftError) -> Self {
		match error {
			MicrosoftError::InvalidGrant | MicrosoftError::Api { status: 403, .. } => {
				VolumeError::PermissionDenied(error.to_string())
			}
			other => VolumeError::Service(other.to_string()),
		}
	}
}

pub type MicrosoftResult<T> = Result<T, MicrosoftError>;
