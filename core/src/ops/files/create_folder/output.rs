//! Output types for create folder operations

use crate::domain::addressing::WingPath;
use crate::infra::job::handle::JobReceipt;
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

/// Output from creating a folder
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateFolderOutput {
	/// Path to the created folder
	pub folder_path: WingPath,
	/// Job receipt if items were moved into the folder
	#[serde(skip_serializing_if = "Option::is_none")]
	pub job_receipt: Option<JobReceipt>,
	/// Operation that `files.undo` takes to remove the new folder again.
	/// Set for an empty folder on this device whose creation was journaled.
	#[serde(default)]
	pub undo_id: Option<Uuid>,
}

impl CreateFolderOutput {
	/// Create output for a folder created without moving items
	pub fn without_items(folder_path: WingPath) -> Self {
		Self {
			folder_path,
			job_receipt: None,
			undo_id: None,
		}
	}

	/// Create output for a folder created with items being moved
	pub fn with_items(folder_path: WingPath, job_receipt: JobReceipt) -> Self {
		Self {
			folder_path,
			job_receipt: Some(job_receipt),
			undo_id: None,
		}
	}

	/// Adds the undo id of the folder's creation.
	pub fn with_undo(mut self, undo_id: Option<Uuid>) -> Self {
		self.undo_id = undo_id;
		self
	}
}
