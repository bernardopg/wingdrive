use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Represents an application that can open a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenWithApp {
	/// Platform-specific identifier:
	/// - macOS: bundle ID (com.apple.Preview)
	/// - Windows: application name
	/// - Linux: desktop entry ID (org.gnome.Evince.desktop)
	pub id: String,

	/// Human-readable display name
	pub name: String,

	/// Optional app icon as a `data:` URL holding base64 PNG or SVG
	/// (filled on Linux).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon: Option<String>,

	/// The user's or system's configured default for this file type
	#[serde(default)]
	pub is_default: bool,
}

/// Result of attempting to open a file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OpenResult {
	Success,
	FileNotFound { path: String },
	AppNotFound { app_id: String },
	PermissionDenied { path: String },
	PlatformError { message: String },
}

/// Trait for platform-specific file opening implementations
pub trait FileOpener: Send + Sync {
	/// Get list of applications that can open this file
	fn get_apps_for_file(&self, path: &Path) -> Result<Vec<OpenWithApp>, String>;

	/// Get list of apps that can open all provided files (intersection)
	///
	/// Keeps the first file's order, which platforms use to put the default
	/// application first.
	fn get_apps_for_files(&self, paths: &[PathBuf]) -> Result<Vec<OpenWithApp>, String> {
		let Some((first, rest)) = paths.split_first() else {
			return Ok(vec![]);
		};
		let mut apps = self.get_apps_for_file(first)?;
		for path in rest {
			let ids = self
				.get_apps_for_file(path)?
				.into_iter()
				.map(|app| app.id)
				.collect::<HashSet<_>>();
			apps.retain(|app| ids.contains(&app.id));
		}
		Ok(apps)
	}

	/// Make `app_id` the default application for files of `path`'s type.
	fn set_default_app(&self, _path: &Path, _app_id: &str) -> Result<(), String> {
		Err("Changing the default application is not supported on this platform".to_string())
	}

	/// Open file with system default application
	fn open_with_default(&self, path: &Path) -> Result<OpenResult, String>;

	/// Open file with specific application
	fn open_with_app(&self, path: &Path, app_id: &str) -> Result<OpenResult, String>;

	/// Open multiple files with specific application
	fn open_files_with_app(
		&self,
		paths: &[PathBuf],
		app_id: &str,
	) -> Result<Vec<OpenResult>, String> {
		paths
			.iter()
			.map(|path| self.open_with_app(path, app_id))
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Apps keyed by extension: `.txt` and `.md` share two editors.
	struct FakeOpener;

	impl FileOpener for FakeOpener {
		fn get_apps_for_file(&self, path: &Path) -> Result<Vec<OpenWithApp>, String> {
			let ids: &[&str] = match path.extension().and_then(|e| e.to_str()) {
				Some("txt") => &["gedit", "kate", "vim"],
				Some("md") => &["obsidian", "vim", "kate"],
				Some("png") => &["gimp"],
				_ => return Err(format!("unknown {}", path.display())),
			};
			Ok(ids
				.iter()
				.map(|id| OpenWithApp {
					id: id.to_string(),
					name: id.to_string(),
					icon: None,
					is_default: false,
				})
				.collect())
		}

		fn open_with_default(&self, _: &Path) -> Result<OpenResult, String> {
			Ok(OpenResult::Success)
		}

		fn open_with_app(&self, _: &Path, _: &str) -> Result<OpenResult, String> {
			Ok(OpenResult::Success)
		}
	}

	fn ids(paths: &[&str]) -> Result<Vec<String>, String> {
		let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
		Ok(FakeOpener
			.get_apps_for_files(&paths)?
			.into_iter()
			.map(|app| app.id)
			.collect())
	}

	#[test]
	fn intersection_keeps_first_files_order_and_shared_apps_only() {
		assert_eq!(ids(&[]).unwrap(), Vec::<String>::new());
		assert_eq!(ids(&["a.txt"]).unwrap(), ["gedit", "kate", "vim"]);
		assert_eq!(ids(&["a.txt", "b.md"]).unwrap(), ["kate", "vim"]);
		assert_eq!(ids(&["a.txt", "b.png"]).unwrap(), Vec::<String>::new());
		assert!(ids(&["a.txt", "b.zzz"]).is_err());
	}
}
