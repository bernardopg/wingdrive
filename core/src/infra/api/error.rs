//! API-specific error types and handling
//!
//! Provides clean error types for the API layer that wrap
//! underlying core errors with additional context.

use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

/// Result type for API operations
pub type ApiResult<T> = Result<T, ApiError>;

/// Comprehensive API error type that wraps all possible operation errors
#[derive(Debug, Error, Serialize, Deserialize, Type)]
pub enum ApiError {
	/// Authentication errors
	#[error("Authentication required")]
	Unauthenticated,

	/// Authorization errors
	#[error("Insufficient permissions: {reason}")]
	InsufficientPermissions { reason: String },

	/// Session/context errors
	#[error("No library selected")]
	NoLibrarySelected,

	#[error("Library not found: {library_id}")]
	LibraryNotFound { library_id: String },

	#[error("Invalid session: {reason}")]
	InvalidSession { reason: String },

	/// Input validation errors
	#[error("Invalid input: {details}")]
	InvalidInput { details: String },

	#[error("Missing required field: {field}")]
	MissingRequiredField { field: String },

	/// Operation execution errors
	#[error("Action execution failed: {reason}")]
	ActionExecutionFailed { reason: String },

	#[error("Query execution failed: {reason}")]
	QueryExecutionFailed { reason: String },

	#[error("Job dispatch failed: {reason}")]
	JobDispatchFailed { reason: String },

	/// Resource errors
	#[error("Resource not found: {resource_type} {resource_id}")]
	ResourceNotFound {
		resource_type: String,
		resource_id: String,
	},

	#[error("Resource conflict: {reason}")]
	ResourceConflict { reason: String },

	/// System errors
	#[error("Database error: {details}")]
	DatabaseError { details: String },

	#[error("Network error: {details}")]
	NetworkError { details: String },

	#[error("File system error: {details}")]
	FileSystemError { details: String },

	/// Rate limiting and quotas
	#[error("Rate limit exceeded: {retry_after_seconds}s")]
	RateLimitExceeded { retry_after_seconds: u64 },

	#[error("Quota exceeded: {quota_type}")]
	QuotaExceeded { quota_type: String },

	/// Generic errors
	#[error("Internal error: {details}")]
	Internal { details: String },

	#[error("Operation timeout")]
	Timeout,
}

impl ApiError {
	/// Create an invalid input error
	pub fn invalid_input<S: Into<String>>(details: S) -> Self {
		Self::InvalidInput {
			details: details.into(),
		}
	}

	/// Create a permission error
	pub fn insufficient_permissions<S: Into<String>>(reason: S) -> Self {
		Self::InsufficientPermissions {
			reason: reason.into(),
		}
	}

	/// Create a resource not found error
	pub fn resource_not_found<T: Into<String>, I: Into<String>>(
		resource_type: T,
		resource_id: I,
	) -> Self {
		Self::ResourceNotFound {
			resource_type: resource_type.into(),
			resource_id: resource_id.into(),
		}
	}

	/// Get the HTTP status code equivalent (useful for REST APIs)
	pub fn status_code(&self) -> u16 {
		match self {
			Self::Unauthenticated => 401,
			Self::InsufficientPermissions { .. } => 403,
			Self::NoLibrarySelected | Self::LibraryNotFound { .. } => 404,
			Self::InvalidInput { .. } | Self::MissingRequiredField { .. } => 400,
			Self::ResourceNotFound { .. } => 404,
			Self::ResourceConflict { .. } => 409,
			Self::RateLimitExceeded { .. } => 429,
			Self::QuotaExceeded { .. } => 429,
			Self::Timeout => 408,
			_ => 500,
		}
	}

	/// Check if this is a client error (4xx) vs server error (5xx)
	pub fn is_client_error(&self) -> bool {
		self.status_code() < 500
	}
}

/// Convert from core action errors
impl From<crate::infra::action::error::ActionError> for ApiError {
	fn from(err: crate::infra::action::error::ActionError) -> Self {
		use crate::infra::action::error::ActionError;
		match err {
			ActionError::LibraryNotFound(id) => ApiError::LibraryNotFound {
				library_id: id.to_string(),
			},
			ActionError::PermissionDenied { reason, .. } => {
				ApiError::InsufficientPermissions { reason }
			}
			ActionError::InvalidInput(details) => ApiError::InvalidInput { details },
			ActionError::Validation { field, message } => ApiError::InvalidInput {
				details: format!("{}: {}", field, message),
			},
			ActionError::Timeout => ApiError::Timeout,
			_ => ApiError::ActionExecutionFailed {
				reason: err.to_string(),
			},
		}
	}
}

/// Convert from core query errors
impl From<crate::infra::query::error::QueryError> for ApiError {
	fn from(err: crate::infra::query::error::QueryError) -> Self {
		use crate::infra::query::error::QueryError;
		match err {
			QueryError::LibraryNotFound(id) => ApiError::LibraryNotFound {
				library_id: id.to_string(),
			},
			QueryError::PermissionDenied { reason, .. } => {
				ApiError::InsufficientPermissions { reason }
			}
			QueryError::InvalidInput(details) => ApiError::InvalidInput { details },
			QueryError::Validation { field, message } => ApiError::InvalidInput {
				details: format!("{}: {}", field, message),
			},
			QueryError::Timeout => ApiError::Timeout,
			_ => ApiError::QueryExecutionFailed {
				reason: err.to_string(),
			},
		}
	}
}

/// Convert from job errors
impl From<crate::infra::job::error::JobError> for ApiError {
	fn from(err: crate::infra::job::error::JobError) -> Self {
		Self::JobDispatchFailed {
			reason: err.to_string(),
		}
	}
}

/// Convert from anyhow errors (common in queries)
impl From<anyhow::Error> for ApiError {
	fn from(err: anyhow::Error) -> Self {
		Self::Internal {
			details: err.to_string(),
		}
	}
}

/// Convert from string errors (common in registry handlers)
impl From<String> for ApiError {
	fn from(err: String) -> Self {
		Self::Internal { details: err }
	}
}

/// Convert from &str errors
impl From<&str> for ApiError {
	fn from(err: &str) -> Self {
		Self::Internal {
			details: err.to_string(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::infra::{action::error::ActionError, query::error::QueryError};

	fn assert_contract(error: ApiError, status: u16, payload: serde_json::Value) {
		assert_eq!(error.status_code(), status);
		assert_eq!(error.is_client_error(), status < 500);
		let encoded = serde_json::to_value(error).unwrap();
		assert_eq!(encoded, payload);
		let decoded: ApiError = serde_json::from_value(encoded).unwrap();
		assert_eq!(decoded.status_code(), status);
	}

	#[test]
	fn action_failures_preserve_public_codes_and_validation_details() {
		use serde_json::json;

		for (source, status, payload) in [
			(
				ActionError::LibraryNotFound(uuid::Uuid::nil()),
				404,
				json!({"LibraryNotFound": {"library_id": uuid::Uuid::nil().to_string()}}),
			),
			(
				ActionError::PermissionDenied {
					action: "files.createFile".into(),
					reason: "read only".into(),
				},
				403,
				json!({"InsufficientPermissions": {"reason": "read only"}}),
			),
			(
				ActionError::InvalidInput("bad payload".into()),
				400,
				json!({"InvalidInput": {"details": "bad payload"}}),
			),
			(
				ActionError::Validation {
					field: "name".into(),
					message: "empty".into(),
				},
				400,
				json!({"InvalidInput": {"details": "name: empty"}}),
			),
			(ActionError::Timeout, 408, json!("Timeout")),
			(
				ActionError::Internal("disk".into()),
				500,
				json!({"ActionExecutionFailed": {"reason": "Internal error: disk"}}),
			),
		] {
			assert_contract(source.into(), status, payload);
		}
	}

	#[test]
	fn query_failures_preserve_public_codes_and_validation_details() {
		use serde_json::json;

		for (source, status, payload) in [
			(
				QueryError::LibraryNotFound(uuid::Uuid::nil()),
				404,
				json!({"LibraryNotFound": {"library_id": uuid::Uuid::nil().to_string()}}),
			),
			(
				QueryError::PermissionDenied {
					query: "files.byId".into(),
					reason: "private".into(),
				},
				403,
				json!({"InsufficientPermissions": {"reason": "private"}}),
			),
			(
				QueryError::InvalidInput("bad payload".into()),
				400,
				json!({"InvalidInput": {"details": "bad payload"}}),
			),
			(
				QueryError::Validation {
					field: "path".into(),
					message: "missing".into(),
				},
				400,
				json!({"InvalidInput": {"details": "path: missing"}}),
			),
			(QueryError::Timeout, 408, json!("Timeout")),
			(
				QueryError::Internal("disk".into()),
				500,
				json!({"QueryExecutionFailed": {"reason": "Internal error: disk"}}),
			),
		] {
			assert_contract(source.into(), status, payload);
		}
	}
}
