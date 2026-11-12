//! Filename validation for rename operations

use std::path::Path;
use thiserror::Error;

/// Errors that can occur during filename validation
#[derive(Debug, Clone, Error)]
pub enum FilenameValidationError {
	#[error("Filename cannot be empty")]
	Empty,

	#[error("Filename cannot contain path separators (/ or \\)")]
	ContainsPathSeparator,

	#[error("Filename cannot be '.' or '..'")]
	InvalidDotName,

	#[error("Filename contains invalid character: {0}")]
	InvalidCharacter(char),

	#[error("Filename is a Windows reserved name: {0}")]
	WindowsReservedName(String),

	#[error("Filename cannot end with a space or period")]
	InvalidEnding,

	#[error("Filename exceeds the maximum length of {0} {unit}", unit = NAME_LENGTH_UNIT)]
	TooLong(usize),
}

/// Windows reserved device names (case-insensitive)
const WINDOWS_RESERVED_NAMES: &[&str] = &[
	"CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
	"COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Invalid characters for filenames across platforms
const INVALID_CHARS: &[char] = &['<', '>', ':', '"', '|', '?', '*', '\0'];

/// Maximum filename length. Linux and macOS file systems limit a name to 255
/// bytes; Windows limits it to 255 UTF-16 code units.
const MAX_FILENAME_LENGTH: usize = 255;

#[cfg(windows)]
const NAME_LENGTH_UNIT: &str = "UTF-16 code units";
#[cfg(not(windows))]
const NAME_LENGTH_UNIT: &str = "bytes";

/// Length of `name` in the unit the platform's file systems limit.
fn name_length(name: &str) -> usize {
	if cfg!(windows) {
		name.encode_utf16().count()
	} else {
		name.len()
	}
}

/// Validate a filename for use in rename operations
///
/// Checks for:
/// - Empty names
/// - Path separators (/ or \)
/// - Invalid special names (. and ..)
/// - Invalid characters (platform-specific)
/// - Windows reserved names (on Windows)
/// - Invalid endings (space or period on Windows)
/// - Maximum length
pub fn validate_filename(name: &str) -> Result<(), FilenameValidationError> {
	// Check for empty name
	if name.is_empty() {
		return Err(FilenameValidationError::Empty);
	}

	// Check for path separators
	if name.contains('/') || name.contains('\\') {
		return Err(FilenameValidationError::ContainsPathSeparator);
	}

	// Check for . and ..
	if name == "." || name == ".." {
		return Err(FilenameValidationError::InvalidDotName);
	}

	// Check for invalid characters
	for c in INVALID_CHARS {
		if name.contains(*c) {
			return Err(FilenameValidationError::InvalidCharacter(*c));
		}
	}

	// Check maximum length
	if name_length(name) > MAX_FILENAME_LENGTH {
		return Err(FilenameValidationError::TooLong(MAX_FILENAME_LENGTH));
	}

	// Platform-specific validation
	#[cfg(target_os = "windows")]
	{
		// Check for Windows reserved names
		let name_upper = name.to_uppercase();
		let base_name = Path::new(&name_upper)
			.file_stem()
			.and_then(|s| s.to_str())
			.unwrap_or(&name_upper);

		if WINDOWS_RESERVED_NAMES.contains(&base_name) {
			return Err(FilenameValidationError::WindowsReservedName(
				base_name.to_string(),
			));
		}

		// Check for invalid endings (Windows doesn't allow trailing space or period)
		if name.ends_with(' ') || name.ends_with('.') {
			return Err(FilenameValidationError::InvalidEnding);
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_valid_filenames() {
		assert!(validate_filename("file.txt").is_ok());
		assert!(validate_filename("my document.pdf").is_ok());
		assert!(validate_filename(".hidden").is_ok());
		assert!(validate_filename("file-name_123").is_ok());
		assert!(validate_filename("日本語ファイル.txt").is_ok());
	}

	#[test]
	fn test_empty_filename() {
		assert!(matches!(
			validate_filename(""),
			Err(FilenameValidationError::Empty)
		));
	}

	#[test]
	fn test_path_separators() {
		assert!(matches!(
			validate_filename("path/to/file"),
			Err(FilenameValidationError::ContainsPathSeparator)
		));
		assert!(matches!(
			validate_filename("path\\to\\file"),
			Err(FilenameValidationError::ContainsPathSeparator)
		));
	}

	#[test]
	fn test_dot_names() {
		assert!(matches!(
			validate_filename("."),
			Err(FilenameValidationError::InvalidDotName)
		));
		assert!(matches!(
			validate_filename(".."),
			Err(FilenameValidationError::InvalidDotName)
		));
	}

	#[test]
	fn test_invalid_characters() {
		assert!(matches!(
			validate_filename("file<name"),
			Err(FilenameValidationError::InvalidCharacter('<'))
		));
		assert!(matches!(
			validate_filename("file:name"),
			Err(FilenameValidationError::InvalidCharacter(':'))
		));
	}

	#[test]
	fn test_too_long() {
		let long_name = "a".repeat(256);
		assert!(matches!(
			validate_filename(&long_name),
			Err(FilenameValidationError::TooLong(_))
		));
		assert!(validate_filename(&"a".repeat(255)).is_ok());
	}

	/// 128 `é` take 256 bytes but 128 UTF-16 units: too long on Linux and macOS,
	/// valid on Windows.
	#[test]
	fn length_limit_uses_the_platform_unit() {
		let accented = "é".repeat(128);
		let result = validate_filename(&accented);
		if cfg!(windows) {
			assert!(result.is_ok());
		} else {
			let error = result.unwrap_err();
			assert!(matches!(error, FilenameValidationError::TooLong(255)));
			assert_eq!(
				error.to_string(),
				"Filename exceeds the maximum length of 255 bytes"
			);
		}
	}

	#[test]
	fn every_forbidden_character_is_rejected_even_inside_unicode_names() {
		for character in ['<', '>', ':', '"', '|', '?', '*', '\0'] {
			let name = format!("café{character}文件.txt");
			assert!(
				matches!(validate_filename(&name), Err(FilenameValidationError::InvalidCharacter(found)) if found == character),
				"{name:?}"
			);
		}
	}

	#[test]
	fn supplementary_unicode_characters_obey_the_exact_platform_boundary() {
		let boundary = if cfg!(windows) {
			format!("{}a", "\u{1f680}".repeat(127))
		} else {
			format!("{}abc", "\u{1f680}".repeat(63))
		};
		assert!(validate_filename(&boundary).is_ok());
		assert!(matches!(
			validate_filename(&format!("{boundary}a")),
			Err(FilenameValidationError::TooLong(255))
		));
	}

	macro_rules! filename_case {
		($name:ident, $input:expr, $valid:expr) => {
			#[test]
			fn $name() {
				assert_eq!(validate_filename($input).is_ok(), $valid, "{:?}", $input);
			}
		};
	}
	filename_case!(filename_plain, "report", true);
	filename_case!(filename_extension, "report.pdf", true);
	filename_case!(filename_hidden, ".hidden", true);
	filename_case!(filename_unicode, "日本語.txt", true);
	filename_case!(filename_emoji, "photo📷.jpg", true);
	filename_case!(filename_internal_space, "my report.pdf", true);
	filename_case!(filename_multiple_dots, "archive.tar.gz", true);
	filename_case!(filename_leading_dash, "-report", true);
	filename_case!(filename_underscore, "report_2026", true);
	filename_case!(filename_double_dot_prefix, "..backup", true);
	filename_case!(filename_empty, "", false);
	filename_case!(filename_current_directory, ".", false);
	filename_case!(filename_parent_directory, "..", false);
	filename_case!(filename_forward_slash, "dir/file", false);
	filename_case!(filename_backslash, "dir\\file", false);
	filename_case!(filename_null, "a\0b", false);
	filename_case!(filename_less_than, "a<b", false);
	filename_case!(filename_greater_than, "a>b", false);
	filename_case!(filename_colon, "a:b", false);
	filename_case!(filename_quote, "a\"b", false);
	filename_case!(filename_pipe, "a|b", false);
	filename_case!(filename_question, "a?b", false);
	filename_case!(filename_star, "a*b", false);
	filename_case!(filename_too_long, &"a".repeat(256), false);
	filename_case!(filename_exact_limit, &"a".repeat(255), true);
}
