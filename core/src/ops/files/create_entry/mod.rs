//! # Create files and links
//!
//! `files.createFile` makes an empty file and `files.createSymlink` a symbolic
//! link, the two entries a file manager creates besides folders. Both refuse
//! to replace an existing name, because the frontend picks a free name from
//! a listing that can be stale by the time the action runs. Only paths on
//! this device are accepted; remote devices need their own daemon to write.
//!
//! ## Example
//! ```rust,ignore
//! let input = CreateFileInput { parent: WingPath::local("/home/me"), name: "notes.txt".into() };
//! let output = CreateFileAction::from_input(input)?.execute(library, context).await?;
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::context::CoreContext;
use crate::domain::addressing::WingPath;
use crate::infra::action::{error::ActionError, LibraryAction, ValidationResult};
use crate::ops::files::rename::validation::validate_filename;

/// Input for creating an empty file
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateFileInput {
	/// Directory that receives the file
	pub parent: WingPath,
	/// Name of the new file
	pub name: String,
}

/// Output from creating a file or link
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateEntryOutput {
	/// Path of the created entry
	pub path: WingPath,
}

/// Input for creating a symbolic link
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateSymlinkInput {
	/// Existing file or folder the link points to
	pub target: WingPath,
	/// Directory that receives the link
	pub parent: WingPath,
	/// Name of the new link
	pub name: String,
}

/// Action for creating an empty file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFileAction {
	input: CreateFileInput,
}

/// Action for creating a symbolic link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSymlinkAction {
	input: CreateSymlinkInput,
}

fn local_dir<'a>(path: &'a WingPath, field: &str) -> Result<&'a Path, ActionError> {
	let invalid = |message: &str| ActionError::Validation {
		field: field.to_string(),
		message: message.to_string(),
	};
	if !path.is_local() {
		return Err(invalid("Only folders on this device can be written"));
	}
	path.as_local_path()
		.ok_or_else(|| invalid("Only folders on this device can be written"))
}

fn validate_name(name: &str) -> Result<(), ActionError> {
	validate_filename(name).map_err(|e| ActionError::Validation {
		field: "name".to_string(),
		message: e.to_string(),
	})
}

fn creation_error(error: std::io::Error, path: &Path) -> ActionError {
	if error.kind() == std::io::ErrorKind::AlreadyExists {
		ActionError::Validation {
			field: "name".to_string(),
			message: format!("{} already exists", path.display()),
		}
	} else {
		ActionError::Internal(format!("Failed to create {}: {error}", path.display()))
	}
}

/// Creates `path` as an empty file, failing if anything already has that name.
pub async fn create_empty_file(path: &Path) -> Result<(), ActionError> {
	tokio::fs::OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(path)
		.await
		.map(|_| ())
		.map_err(|e| creation_error(e, path))
}

/// Creates a symbolic link at `link` pointing to `target`.
pub async fn create_link(target: &Path, link: &Path) -> Result<(), ActionError> {
	#[cfg(unix)]
	let result = tokio::fs::symlink(target, link).await;
	#[cfg(windows)]
	let result = if tokio::fs::metadata(target).await.is_ok_and(|m| m.is_dir()) {
		tokio::fs::symlink_dir(target, link).await
	} else {
		tokio::fs::symlink_file(target, link).await
	};
	result.map_err(|e| creation_error(e, link))
}

impl LibraryAction for CreateFileAction {
	type Input = CreateFileInput;
	type Output = CreateEntryOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn validate(
		&self,
		_library: &Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<ValidationResult, ActionError> {
		validate_name(&self.input.name)?;
		local_dir(&self.input.parent, "parent")?;
		Ok(ValidationResult::Success { metadata: None })
	}

	async fn execute(
		self,
		_library: Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let parent = local_dir(&self.input.parent, "parent")?;
		create_empty_file(&parent.join(&self.input.name)).await?;
		Ok(CreateEntryOutput {
			path: self.input.parent.join(&self.input.name),
		})
	}

	fn action_kind(&self) -> &'static str {
		"files.createFile"
	}
}

impl LibraryAction for CreateSymlinkAction {
	type Input = CreateSymlinkInput;
	type Output = CreateEntryOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn validate(
		&self,
		_library: &Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<ValidationResult, ActionError> {
		validate_name(&self.input.name)?;
		local_dir(&self.input.parent, "parent")?;
		let target = local_dir(&self.input.target, "target")?;
		if tokio::fs::symlink_metadata(target).await.is_err() {
			return Err(ActionError::Validation {
				field: "target".to_string(),
				message: format!("{} does not exist", target.display()),
			});
		}
		Ok(ValidationResult::Success { metadata: None })
	}

	async fn execute(
		self,
		_library: Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let parent = local_dir(&self.input.parent, "parent")?;
		let target: PathBuf = local_dir(&self.input.target, "target")?.to_path_buf();
		create_link(&target, &parent.join(&self.input.name)).await?;
		Ok(CreateEntryOutput {
			path: self.input.parent.join(&self.input.name),
		})
	}

	fn action_kind(&self) -> &'static str {
		"files.createSymlink"
	}
}

crate::register_library_action!(CreateFileAction, "files.createFile");
crate::register_library_action!(CreateSymlinkAction, "files.createSymlink");

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn empty_file_is_created_once() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("Untitled File");
		create_empty_file(&path).await.unwrap();
		assert_eq!(std::fs::metadata(&path).unwrap().len(), 0);

		std::fs::write(&path, "kept").unwrap();
		let error = create_empty_file(&path).await.unwrap_err();
		assert!(matches!(error, ActionError::Validation { .. }), "{error:?}");
		assert_eq!(std::fs::read_to_string(&path).unwrap(), "kept");
	}

	#[cfg(unix)]
	#[tokio::test]
	async fn link_points_to_target_and_never_replaces() {
		let dir = tempfile::tempdir().unwrap();
		let target = dir.path().join("docs");
		std::fs::create_dir(&target).unwrap();
		let link = dir.path().join("docs (link)");
		create_link(&target, &link).await.unwrap();
		assert_eq!(std::fs::read_link(&link).unwrap(), target);

		let error = create_link(&target, &link).await.unwrap_err();
		assert!(matches!(error, ActionError::Validation { .. }), "{error:?}");
	}

	#[test]
	fn remote_paths_are_rejected() {
		let remote = WingPath::Physical {
			device_slug: "another-device".into(),
			path: "/home/me".into(),
		};
		assert!(local_dir(&remote, "parent").is_err());
		assert!(local_dir(&WingPath::local("/home/me"), "parent").is_ok());
	}
}
