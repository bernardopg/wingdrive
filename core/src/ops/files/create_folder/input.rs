//! Input types for create folder operations

use crate::domain::addressing::WingPath;
use serde::{Deserialize, Serialize};
use specta::Type;

/// Input for creating a new folder
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateFolderInput {
	/// Parent directory where the folder will be created
	pub parent: WingPath,
	/// Name for the new folder
	pub name: String,
	/// Optional items to move into the new folder after creation
	#[serde(default)]
	pub items: Vec<WingPath>,
}

impl CreateFolderInput {
	/// Create a new folder input without items
	pub fn new(parent: WingPath, name: impl Into<String>) -> Self {
		Self {
			parent,
			name: name.into(),
			items: Vec::new(),
		}
	}

	/// Create a new folder input with items to move into it
	pub fn with_items(parent: WingPath, name: impl Into<String>, items: Vec<WingPath>) -> Self {
		Self {
			parent,
			name: name.into(),
			items,
		}
	}
}
