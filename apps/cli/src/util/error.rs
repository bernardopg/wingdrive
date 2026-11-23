use std::fmt;

/// Custom error type for CLI operations
#[derive(Debug)]
pub enum CliError {
	/// No active library selected
	NoActiveLibrary,
	/// Library not found
	LibraryNotFound(uuid::Uuid),
	/// Location not found
	LocationNotFound(uuid::Uuid),
	/// Multiple libraries exist but no specific one selected
	MultipleLibraries,
	/// Daemon is not running
	DaemonNotRunning,
	/// Core operation failed
	CoreError(String),
	/// Serialization/deserialization error
	SerializationError(String),
	/// Other error
	Other(String),
}

impl fmt::Display for CliError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
            Self::NoActiveLibrary => write!(f, "No active library selected"),
            Self::LibraryNotFound(id) => write!(f, "Library not found: {}", id),
            Self::LocationNotFound(id) => write!(f, "Location not found: {}", id),
            Self::MultipleLibraries => write!(
                f,
                "Multiple libraries exist. Please specify one with --library or switch to it with 'library switch'"
            ),
            Self::DaemonNotRunning => {
                write!(f, "WingDrive daemon is not running\n\n")?;
                write!(f, "To start the daemon, run:\n")?;
                write!(f, "   wing start\n\n")?;
                write!(f, "   Or start with networking enabled:\n")?;
                write!(f, "   wing start")
            },
            Self::CoreError(msg) => write!(f, "Core operation failed: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
	}
}

impl std::error::Error for CliError {}

impl From<anyhow::Error> for CliError {
	fn from(err: anyhow::Error) -> Self {
		Self::Other(err.to_string())
	}
}

/// Result type for CLI operations
pub type CliResult<T> = Result<T, CliError>;

/// Check if an error message indicates the daemon is not running
pub fn is_daemon_connection_error(error_msg: &str) -> bool {
	error_msg.contains("Failed to connect to daemon socket")
		|| error_msg.contains("Connection refused")
		|| error_msg.contains("No such file or directory")
		|| error_msg.contains("daemon socket")
}

/// Convert a core error to a more user-friendly CLI error
pub fn improve_core_error(error_msg: String) -> CliError {
	if is_daemon_connection_error(&error_msg) {
		CliError::DaemonNotRunning
	} else {
		CliError::CoreError(error_msg)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn cli_error_display_formats_all_variants() {
		let id = uuid::Uuid::nil();
		let cases: Vec<(CliError, &[&str])> = vec![
			(CliError::NoActiveLibrary, &["No active library selected"]),
			(
				CliError::LibraryNotFound(id),
				&["Library not found:", "00000000-0000-0000-0000-000000000000"],
			),
			(
				CliError::LocationNotFound(id),
				&[
					"Location not found:",
					"00000000-0000-0000-0000-000000000000",
				],
			),
			(
				CliError::MultipleLibraries,
				&["Multiple libraries exist", "library switch"],
			),
			(
				CliError::DaemonNotRunning,
				&["daemon is not running", "wing start"],
			),
			(
				CliError::CoreError("boom".to_string()),
				&["Core operation failed: boom"],
			),
			(
				CliError::SerializationError("bad json".to_string()),
				&["Serialization error: bad json"],
			),
			(CliError::Other("plain".to_string()), &["plain"]),
		];

		for (error, needles) in cases {
			let message = error.to_string();
			for needle in needles {
				assert!(
					message.contains(needle),
					"message {message:?} did not contain {needle:?}"
				);
			}
		}
	}

	#[test]
	fn daemon_connection_error_detection_is_specific() {
		for message in [
			"Failed to connect to daemon socket at /tmp/x",
			"Connection refused (os error 111)",
			"No such file or directory (os error 2)",
			"daemon socket missing",
		] {
			assert!(
				is_daemon_connection_error(message),
				"expected daemon error for {message:?}"
			);
		}

		for message in [
			"Permission denied",
			"socket timeout",
			"connection reset by peer",
			"",
		] {
			assert!(
				!is_daemon_connection_error(message),
				"unexpected daemon error for {message:?}"
			);
		}
	}

	#[test]
	fn improve_core_error_maps_connection_and_preserves_other_errors() {
		assert!(matches!(
			improve_core_error("Connection refused".to_string()),
			CliError::DaemonNotRunning
		));
		match improve_core_error("validation failed".to_string()) {
			CliError::CoreError(message) => assert_eq!(message, "validation failed"),
			other => panic!("expected CoreError, got {other:?}"),
		}
	}

	#[test]
	fn anyhow_error_converts_to_other() {
		let error = CliError::from(anyhow::anyhow!("something went wrong"));
		match error {
			CliError::Other(message) => assert!(message.contains("something went wrong")),
			other => panic!("expected Other, got {other:?}"),
		}
	}
}
