//! File rename action handler

use super::input::FileRenameInput;
use super::validation::validate_filename;
use crate::{
	context::CoreContext,
	domain::addressing::WingPath,
	infra::{
		action::{error::ActionError, LibraryAction, ValidationResult},
		job::handle::JobReceipt,
	},
	ops::files::copy::job::{same_entry, FileCopyJob},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Action for renaming a file or directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRenameAction {
	/// The file or directory to rename
	pub target: WingPath,
	/// The new name (filename only, no path)
	pub new_name: String,
}

impl FileRenameAction {
	/// Create a new rename action
	pub fn new(target: WingPath, new_name: impl Into<String>) -> Self {
		Self {
			target,
			new_name: new_name.into(),
		}
	}
}

impl LibraryAction for FileRenameAction {
	type Input = FileRenameInput;
	type Output = JobReceipt;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(FileRenameAction {
			target: input.target,
			new_name: input.new_name,
		})
	}

	async fn validate(
		&self,
		_library: &Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<ValidationResult, ActionError> {
		// Validate the new filename
		validate_filename(&self.new_name).map_err(|e| ActionError::Validation {
			field: "new_name".to_string(),
			message: e.to_string(),
		})?;

		// Validate target is not a Content or Sidecar path (these cannot be renamed directly)
		match &self.target {
			WingPath::Content { .. } => {
				return Err(ActionError::Validation {
					field: "target".to_string(),
					message: "Cannot rename content-addressed files directly".to_string(),
				});
			}
			WingPath::Sidecar { .. } => {
				return Err(ActionError::Validation {
					field: "target".to_string(),
					message: "Cannot rename sidecar files directly".to_string(),
				});
			}
			_ => {}
		}

		// Refuse a name that is taken before queueing the job, so the person sees
		// the conflict now. A case-only rename on a case-insensitive volume finds
		// the target itself, which is not a conflict.
		if let Some(path) = self.target.as_local_path() {
			if let Some(parent) = path.parent() {
				let destination = WingPath::local(parent.join(&self.new_name));
				let taken = destination
					.as_local_path()
					.is_some_and(|path| std::fs::symlink_metadata(path).is_ok());
				if taken && !same_entry(&self.target, &destination).await {
					return Err(ActionError::Validation {
						field: "new_name".to_string(),
						message: format!("An item named \"{}\" already exists", self.new_name),
					});
				}
			}
		}

		Ok(ValidationResult::Success { metadata: None })
	}

	async fn execute(
		self,
		library: Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		// Use FileCopyJob::new_rename which handles the rename as a move operation
		let job = FileCopyJob::new_rename(self.target, self.new_name);

		let job_handle = library
			.jobs()
			.dispatch(job)
			.await
			.map_err(ActionError::Job)?;

		Ok(job_handle.into())
	}

	fn action_kind(&self) -> &'static str {
		"files.rename"
	}
}

// Register with the action-centric registry
crate::register_library_action!(FileRenameAction, "files.rename");

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_action_creation() {
		let target = WingPath::local(std::path::PathBuf::from("/test/file.txt"));
		let action = FileRenameAction::new(target, "newname.txt");
		assert_eq!(action.new_name, "newname.txt");
	}

	#[test]
	fn test_cloud_rename_targets_the_same_folder() {
		let input = FileRenameInput::new(
			WingPath::Cloud {
				service: crate::volume::backend::CloudServiceType::S3,
				identifier: "bucket".into(),
				path: "docs/file.txt".into(),
			},
			"renamed.txt",
		);
		let action = FileRenameAction::from_input(input).unwrap();
		let job = FileCopyJob::new_rename(action.target, action.new_name);
		assert_eq!(
			job.destination,
			WingPath::Cloud {
				service: crate::volume::backend::CloudServiceType::S3,
				identifier: "bucket".into(),
				path: "docs/renamed.txt".into(),
			}
		);
	}
}
