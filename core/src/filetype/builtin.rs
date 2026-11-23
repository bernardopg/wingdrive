//! Built-in file type definitions
//!
//! Loads the built-in file type definitions from embedded TOML files.

use once_cell::sync::Lazy;

/// Embedded TOML definitions
pub static BUILTIN_DEFINITIONS: Lazy<Vec<&'static str>> = Lazy::new(|| {
	vec![
		include_str!("definitions/images.toml"),
		include_str!("definitions/video.toml"),
		include_str!("definitions/audio.toml"),
		include_str!("definitions/documents.toml"),
		include_str!("definitions/code.toml"),
		include_str!("definitions/archives.toml"),
		include_str!("definitions/spreadsheets.toml"),
		include_str!("definitions/presentations.toml"),
		include_str!("definitions/email.toml"),
		include_str!("definitions/calendar.toml"),
		include_str!("definitions/contacts.toml"),
		include_str!("definitions/web.toml"),
		include_str!("definitions/shortcuts.toml"),
		include_str!("definitions/packages.toml"),
		include_str!("definitions/misc.toml"),
	]
});

/// Get all built-in TOML definitions
pub fn get_builtin_toml_definitions() -> &'static [&'static str] {
	&BUILTIN_DEFINITIONS
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn builtin_definitions_are_loaded() {
		let defs = get_builtin_toml_definitions();
		assert_eq!(defs.len(), 15);
		assert!(defs.iter().all(|d| !d.trim().is_empty()));
	}

	#[test]
	fn builtin_definitions_include_expected_categories() {
		let all = get_builtin_toml_definitions().join("\n");
		for needle in [
			"image",
			"video",
			"audio",
			"document",
			"code",
			"archive",
			"spreadsheet",
			"presentation",
		] {
			assert!(
				all.to_lowercase().contains(needle),
				"missing {needle} in builtin definitions"
			);
		}
	}
}
