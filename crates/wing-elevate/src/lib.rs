//! # WingDrive elevated file operations
//!
//! A small helper runs as administrator to repeat file operations that the
//! user could not perform with normal rights. The desktop app starts it through
//! UAC on Windows; it reads its request from a file and writes a response file,
//! never taking operations from its command line or a shell. Paths are handled
//! as data, so names with shell metacharacters are safe.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// One operation to repeat with administrator rights.
///
/// The shape matches the frontend's `AdminOperation` and DESK-033 on Linux.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdminOperation {
	/// Removes files and folders, recursively.
	Delete { paths: Vec<PathBuf> },
	/// Renames an entry within its folder.
	Rename { path: PathBuf, new_name: String },
	/// Creates an empty folder.
	CreateFolder { parent: PathBuf, name: String },
	/// Copies or moves entries into a folder, skipping names that exist.
	Transfer {
		sources: Vec<PathBuf>,
		destination: PathBuf,
		#[serde(default)]
		move_files: bool,
	},
}

/// Request read by the elevated helper.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AdminRequest {
	pub operation: AdminOperation,
}

/// Result of one file or folder touched by an operation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AdminItemFailure {
	pub path: String,
	pub error: String,
}

/// Outcome reported by the helper.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AdminStatus {
	/// Every requested item completed.
	Completed,
	/// The operation could not start.
	Failed { message: String },
	/// Some items completed and some failed.
	Partial { failures: Vec<AdminItemFailure> },
}

/// Response written by the elevated helper.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AdminResponse {
	pub status: AdminStatus,
}

impl AdminResponse {
	pub fn completed() -> Self {
		Self { status: AdminStatus::Completed }
	}

	pub fn failed(message: impl Into<String>) -> Self {
		Self {
			status: AdminStatus::Failed {
				message: message.into(),
			},
		}
	}

	pub fn from_failures(failures: Vec<AdminItemFailure>) -> Self {
		if failures.is_empty() {
			Self::completed()
		} else {
			Self {
				status: AdminStatus::Partial { failures },
			}
		}
	}
}

/// Runs `operation` and reports per-item failures instead of stopping at the first.
pub fn execute(operation: &AdminOperation) -> AdminResponse {
	match validate(operation) {
		Ok(()) => {}
		Err(message) => return AdminResponse::failed(message),
	}

	match operation {
		AdminOperation::Delete { paths } => execute_delete(paths),
		AdminOperation::Rename { path, new_name } => execute_rename(path, new_name),
		AdminOperation::CreateFolder { parent, name } => execute_create_folder(parent, name),
		AdminOperation::Transfer {
			sources,
			destination,
			move_files,
		} => execute_transfer(sources, destination, *move_files),
	}
}

fn validate(operation: &AdminOperation) -> Result<(), String> {
	match operation {
		AdminOperation::Delete { paths } => {
			require_some(paths)?;
			for path in paths {
				checked_path(path)?;
			}
		}
		AdminOperation::Rename { path, new_name } => {
			checked_path(path)?;
			checked_name(new_name)?;
		}
		AdminOperation::CreateFolder { parent, name } => {
			checked_path(parent)?;
			checked_name(name)?;
		}
		AdminOperation::Transfer {
			sources,
			destination,
			..
		} => {
			require_some(sources)?;
			for source in sources {
				checked_path(source)?;
			}
			checked_path(destination)?;
		}
	}
	Ok(())
}

fn require_some(paths: &[PathBuf]) -> Result<(), String> {
	if paths.is_empty() {
		Err("No files were given".into())
	} else {
		Ok(())
	}
}

/// A path that is absolute and not the filesystem root.
///
/// The check is intentionally conservative: it rejects relative paths and roots,
/// but it does not interpret shell syntax because operations never pass through
/// a shell.
fn checked_path(path: &Path) -> Result<(), String> {
	if path.as_os_str().is_empty() {
		return Err("The path is empty".into());
	}
	if !path.is_absolute() {
		return Err(format!("{} is not an absolute path", path.display()));
	}
	if path.parent().is_none() {
		return Err("The filesystem root cannot be changed this way".into());
	}
	Ok(())
}

/// A single file name, not a path.
fn checked_name(name: &str) -> Result<(), String> {
	if name.is_empty() || name == "." || name == ".." {
		return Err(format!("{name:?} is not a valid name"));
	}
	if name.contains('/') || name.contains('\\') || name.contains('\0') {
		return Err(format!("{name:?} is not a valid name"));
	}
	Ok(())
}

fn execute_delete(paths: &[PathBuf]) -> AdminResponse {
	let mut failures = Vec::new();
	for path in paths {
		let result = if path.is_dir() {
			fs::remove_dir_all(path)
		} else {
			fs::remove_file(path)
		};
		if let Err(error) = result {
			failures.push(AdminItemFailure {
				path: path.display().to_string(),
				error: error.to_string(),
			});
		}
	}
	AdminResponse::from_failures(failures)
}

fn execute_rename(path: &Path, new_name: &str) -> AdminResponse {
	let Some(parent) = path.parent() else {
		return AdminResponse::failed("The path has no folder");
	};
	let target = parent.join(new_name);
	match fs::rename(path, &target) {
		Ok(()) => AdminResponse::completed(),
		Err(error) => AdminResponse::from_failures(vec![AdminItemFailure {
			path: path.display().to_string(),
			error: error.to_string(),
		}]),
	}
}

fn execute_create_folder(parent: &Path, name: &str) -> AdminResponse {
	let target = parent.join(name);
	match fs::create_dir(&target) {
		Ok(()) => AdminResponse::completed(),
		Err(error) => AdminResponse::from_failures(vec![AdminItemFailure {
			path: target.display().to_string(),
			error: error.to_string(),
		}]),
	}
}

fn execute_transfer(sources: &[PathBuf], destination: &Path, move_files: bool) -> AdminResponse {
	if !destination.is_dir() {
		return AdminResponse::failed(format!(
			"{} is not a folder",
			destination.display()
		));
	}

	let mut failures = Vec::new();
	for source in sources {
		let Some(name) = source.file_name() else {
			failures.push(AdminItemFailure {
				path: source.display().to_string(),
				error: "The path has no file name".into(),
			});
			continue;
		};
		let target = destination.join(name);
		if target.exists() {
			// `-n` semantics: skip existing names instead of overwriting.
			continue;
		}
		let result = if move_files {
			move_entry(source, &target)
		} else {
			copy_entry(source, &target)
		};
		if let Err(error) = result {
			failures.push(AdminItemFailure {
				path: source.display().to_string(),
				error,
			});
		}
	}
	AdminResponse::from_failures(failures)
}

fn move_entry(source: &Path, target: &Path) -> Result<(), String> {
	match fs::rename(source, target) {
		Ok(()) => Ok(()),
		Err(rename_error) => {
			// Cross-device moves need copy+delete.
			copy_entry(source, target)?;
			let remove = if source.is_dir() {
				fs::remove_dir_all(source)
			} else {
				fs::remove_file(source)
			};
			remove.map_err(|e| {
				format!(
					"copied, but could not remove source after rename failed ({rename_error}): {e}"
				)
			})
		}
	}
}

fn copy_entry(source: &Path, target: &Path) -> Result<(), String> {
	let metadata = fs::symlink_metadata(source).map_err(|e| e.to_string())?;
	let file_type = metadata.file_type();
	if file_type.is_symlink() {
		copy_symlink(source, target)
	} else if file_type.is_dir() {
		copy_dir(source, target)
	} else {
		fs::copy(source, target).map(|_| ()).map_err(|e| e.to_string())
	}
}

fn copy_dir(source: &Path, target: &Path) -> Result<(), String> {
	fs::create_dir(target).map_err(|e| e.to_string())?;
	for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
		let entry = entry.map_err(|e| e.to_string())?;
		let child_source = entry.path();
		let child_target = target.join(entry.file_name());
		if child_target.exists() {
			continue;
		}
		copy_entry(&child_source, &child_target)?;
	}
	Ok(())
}

#[cfg(unix)]
fn copy_symlink(source: &Path, target: &Path) -> Result<(), String> {
	let link = fs::read_link(source).map_err(|e| e.to_string())?;
	std::os::unix::fs::symlink(link, target).map_err(|e| e.to_string())
}

#[cfg(windows)]
fn copy_symlink(source: &Path, target: &Path) -> Result<(), String> {
	let link = fs::read_link(source).map_err(|e| e.to_string())?;
	let metadata = fs::metadata(source);
	let is_dir = metadata.map(|m| m.is_dir()).unwrap_or(false);
	let result = if is_dir {
		std::os::windows::fs::symlink_dir(&link, target)
	} else {
		std::os::windows::fs::symlink_file(&link, target)
	};
	result.map_err(|e| e.to_string())
}

#[cfg(not(any(unix, windows)))]
fn copy_symlink(_source: &Path, _target: &Path) -> Result<(), String> {
	Err("Symlinks are not supported on this platform".into())
}

/// Reads a request from `path`.
pub fn read_request(path: &Path) -> Result<AdminRequest, String> {
	let bytes = fs::read(path).map_err(|e| format!("Failed to read request: {e}"))?;
	serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse request: {e}"))
}

/// Writes a response to `path` as JSON.
pub fn write_response(path: &Path, response: &AdminResponse) -> Result<(), String> {
	let json = serde_json::to_vec(response).map_err(|e| format!("Failed to serialize response: {e}"))?;
	fs::write(path, json).map_err(|e| format!("Failed to write response: {e}"))
}

/// Runs the helper protocol: read request, execute, write response.
pub fn run_request_file(request_path: &Path, response_path: &Path) -> Result<(), String> {
	let response = match read_request(request_path) {
		Ok(request) => execute(&request.operation),
		Err(message) => AdminResponse::failed(message),
	};
	write_response(response_path, &response)
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::tempdir;

	fn write_file(path: &Path, contents: &str) {
		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent).unwrap();
		}
		fs::write(path, contents).unwrap();
	}

	#[test]
	fn request_response_round_trip() {
		let dir = tempdir().unwrap();
		let request_path = dir.path().join("request.json");
		let response_path = dir.path().join("response.json");
		let file = dir.path().join("a.txt");
		write_file(&file, "hello");

		let request = AdminRequest {
			operation: AdminOperation::Delete { paths: vec![file.clone()] },
		};
		fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();

		run_request_file(&request_path, &response_path).unwrap();
		let response: AdminResponse =
			serde_json::from_slice(&fs::read(&response_path).unwrap()).unwrap();
		assert_eq!(response, AdminResponse::completed());
		assert!(!file.exists());
	}

	#[test]
	fn special_characters_are_handled_as_data() {
		let dir = tempdir().unwrap();
		let names = [
			"a&b.txt",
			"a|b.txt",
			"a\"b.txt",
			"a%b.txt",
			"a^b.txt",
			"mix &|\"%^!.txt",
		];
		let mut paths = Vec::new();
		for name in names {
			let path = dir.path().join(name);
			write_file(&path, name);
			paths.push(path);
		}

		let destination = dir.path().join("dest");
		fs::create_dir(&destination).unwrap();

		let response = execute(&AdminOperation::Transfer {
			sources: paths.clone(),
			destination: destination.clone(),
			move_files: false,
		});
		assert_eq!(response, AdminResponse::completed());
		for path in &paths {
			let name = path.file_name().unwrap();
			let copied = destination.join(name);
			assert_eq!(fs::read_to_string(&copied).unwrap(), name.to_string_lossy());
			assert!(path.exists(), "copy should keep source");
		}

		let renamed = destination.join("renamed &|\"%^.txt");
		let first = destination.join(names[0]);
		let response = execute(&AdminOperation::Rename {
			path: first.clone(),
			new_name: renamed.file_name().unwrap().to_string_lossy().into_owned(),
		});
		assert_eq!(response, AdminResponse::completed());
		assert!(!first.exists());
		assert!(renamed.exists());

		let folder = dir.path().join("folder &|\"%^");
		let response = execute(&AdminOperation::CreateFolder {
			parent: dir.path().to_path_buf(),
			name: folder.file_name().unwrap().to_string_lossy().into_owned(),
		});
		assert_eq!(response, AdminResponse::completed());
		assert!(folder.is_dir());

		let delete_targets = vec![renamed, folder];
		let response = execute(&AdminOperation::Delete { paths: delete_targets.clone() });
		assert_eq!(response, AdminResponse::completed());
		for path in delete_targets {
			assert!(!path.exists());
		}
	}

	#[test]
	fn transfer_skips_existing_names() {
		let dir = tempdir().unwrap();
		let src = dir.path().join("src");
		let dest = dir.path().join("dest");
		fs::create_dir(&src).unwrap();
		fs::create_dir(&dest).unwrap();
		write_file(&src.join("a.txt"), "new");
		write_file(&dest.join("a.txt"), "kept");

		let response = execute(&AdminOperation::Transfer {
			sources: vec![src.join("a.txt")],
			destination: dest.clone(),
			move_files: false,
		});
		assert_eq!(response, AdminResponse::completed());
		assert_eq!(fs::read_to_string(dest.join("a.txt")).unwrap(), "kept");
	}

	#[test]
	fn reports_partial_failures_per_item() {
		let dir = tempdir().unwrap();
		let missing = dir.path().join("missing.txt");
		let present = dir.path().join("present.txt");
		write_file(&present, "ok");

		let response = execute(&AdminOperation::Delete {
			paths: vec![missing.clone(), present.clone()],
		});
		match response.status {
			AdminStatus::Partial { failures } => {
				assert_eq!(failures.len(), 1);
				assert_eq!(failures[0].path, missing.display().to_string());
			}
			other => panic!("expected partial, got {other:?}"),
		}
		assert!(!present.exists());
	}

	#[test]
	fn refuses_empty_and_root_paths() {
		let root = PathBuf::from(std::path::MAIN_SEPARATOR.to_string());
		let err = execute(&AdminOperation::Delete { paths: vec![root] });
		assert!(matches!(err.status, AdminStatus::Failed { .. }));

		let err = execute(&AdminOperation::Delete { paths: vec![] });
		assert!(matches!(err.status, AdminStatus::Failed { .. }));

		let err = execute(&AdminOperation::Rename {
			path: PathBuf::from("relative.txt"),
			new_name: "x".into(),
		});
		assert!(matches!(err.status, AdminStatus::Failed { .. }));

		let err = execute(&AdminOperation::CreateFolder {
			parent: PathBuf::from("/tmp"),
			name: "../x".into(),
		});
		assert!(matches!(err.status, AdminStatus::Failed { .. }));
	}
}
