//! File delete action handler

use super::input::FileDeleteInput;
use super::job::{DeleteJob, DeleteOptions};
use crate::{
	context::CoreContext,
	domain::addressing::{WingPath, WingPathBatch},
	infra::{
		action::{error::ActionError, LibraryAction},
		job::handle::JobHandle,
	},
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDeleteAction {
	pub targets: WingPathBatch,
	pub options: DeleteOptions,
}

impl FileDeleteAction {
	/// Create a new file delete action
	pub fn new(targets: WingPathBatch, options: DeleteOptions) -> Self {
		Self { targets, options }
	}

	/// Create a delete action with default options
	pub fn with_defaults(targets: WingPathBatch) -> Self {
		Self::new(targets, DeleteOptions::default())
	}
}

// Implement the unified LibraryAction
impl LibraryAction for FileDeleteAction {
	type Input = FileDeleteInput;
	type Output = crate::infra::job::handle::JobReceipt;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(FileDeleteAction {
			targets: input.targets,
			options: DeleteOptions {
				permanent: input.permanent,
				recursive: input.recursive,
			},
		})
	}

	async fn execute(
		self,
		library: std::sync::Arc<crate::library::Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		// `permanent` arrives only after the client confirmed it (the UI
		// dialog or the CLI prompt), so the action carries that confirmation.
		let job = if self.options.permanent {
			DeleteJob::permanent(self.targets, true)
		} else {
			DeleteJob::trash(self.targets)
		};

		let job_handle = library
			.jobs()
			.dispatch(job)
			.await
			.map_err(ActionError::Job)?;

		Ok(job_handle.into())
	}

	fn action_kind(&self) -> &'static str {
		"files.delete"
	}

	async fn validate(
		&self,
		_library: &std::sync::Arc<crate::library::Library>,
		_context: std::sync::Arc<crate::context::CoreContext>,
	) -> Result<crate::infra::action::ValidationResult, ActionError> {
		// Validate targets
		if self.targets.paths.is_empty() {
			return Err(ActionError::Validation {
				field: "targets".to_string(),
				message: "At least one target file must be specified".to_string(),
			});
		}

		Ok(crate::infra::action::ValidationResult::Success { metadata: None })
	}
}

// Register this action with the new registry
crate::register_library_action!(FileDeleteAction, "files.delete");
