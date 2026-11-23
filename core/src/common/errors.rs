//! Unified error handling for the core

use thiserror::Error;

/// Main error type for core operations
#[derive(Error, Debug)]
pub enum CoreError {
	#[error("Database error: {0}")]
	Database(#[from] sea_orm::DbErr),

	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[error("File operation error: {0}")]
	FileOp(#[from] FileOpError),

	#[error("Not found: {0}")]
	NotFound(String),

	#[error("Invalid operation: {0}")]
	InvalidOperation(String),

	#[error("Other error: {0}")]
	Other(#[from] anyhow::Error),
}

/// Errors specific to file operations
#[derive(Error, Debug)]
pub enum FileOpError {
	#[error("Source not found: {0}")]
	SourceNotFound(String),

	#[error("Destination not found: {0}")]
	DestinationNotFound(String),

	#[error("Permission denied: {0}")]
	PermissionDenied(String),

	#[error("File exists: {0}")]
	FileExists(String),

	#[error("Not a directory: {0}")]
	NotADirectory(String),

	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[error("Other: {0}")]
	Other(String),
}

impl From<&str> for FileOpError {
	fn from(s: &str) -> Self {
		FileOpError::Other(s.to_string())
	}
}

impl From<String> for FileOpError {
	fn from(s: String) -> Self {
		FileOpError::Other(s)
	}
}

/// Result type alias for core operations
pub type Result<T> = std::result::Result<T, CoreError>;

#[cfg(test)]
mod tests {
	use super::*;
	use std::io;

	#[test]
	fn core_error_display() {
		let db_err = CoreError::Database(sea_orm::DbErr::Custom("db".to_string()));
		assert_eq!(db_err.to_string(), "Database error: Custom Error: db");

		let io_err = CoreError::Io(io::Error::other("io"));
		assert_eq!(io_err.to_string(), "IO error: io");

		let file_err = CoreError::FileOp(FileOpError::FileExists("f".to_string()));
		assert_eq!(file_err.to_string(), "File operation error: File exists: f");

		let not_found = CoreError::NotFound("x".to_string());
		assert_eq!(not_found.to_string(), "Not found: x");

		let invalid = CoreError::InvalidOperation("bad".to_string());
		assert_eq!(invalid.to_string(), "Invalid operation: bad");

		let other = CoreError::Other(anyhow::anyhow!("boom"));
		assert_eq!(other.to_string(), "Other error: boom");
	}

	#[test]
	fn file_op_error_display() {
		assert_eq!(
			FileOpError::SourceNotFound("a".to_string()).to_string(),
			"Source not found: a"
		);
		assert_eq!(
			FileOpError::DestinationNotFound("b".to_string()).to_string(),
			"Destination not found: b"
		);
		assert_eq!(
			FileOpError::PermissionDenied("c".to_string()).to_string(),
			"Permission denied: c"
		);
		assert_eq!(
			FileOpError::FileExists("d".to_string()).to_string(),
			"File exists: d"
		);
		assert_eq!(
			FileOpError::NotADirectory("e".to_string()).to_string(),
			"Not a directory: e"
		);
		assert_eq!(
			FileOpError::Io(io::Error::other("io")).to_string(),
			"IO error: io"
		);
		assert_eq!(FileOpError::Other("z".to_string()).to_string(), "Other: z");
	}

	#[test]
	fn file_op_error_from_str_and_string() {
		let from_str: FileOpError = "msg".into();
		assert!(matches!(from_str, FileOpError::Other(s) if s == "msg"));

		let from_string: FileOpError = String::from("owned").into();
		assert!(matches!(from_string, FileOpError::Other(s) if s == "owned"));
	}

	#[test]
	fn core_error_from_conversions() {
		let io_err = io::Error::other("io");
		let core: CoreError = io_err.into();
		assert!(matches!(core, CoreError::Io(_)));

		let file_err = FileOpError::Other("x".to_string());
		let core: CoreError = file_err.into();
		assert!(matches!(core, CoreError::FileOp(_)));

		let any = anyhow::anyhow!("any");
		let core: CoreError = any.into();
		assert!(matches!(core, CoreError::Other(_)));
	}
}
