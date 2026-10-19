//! Location remove action handler

use super::output::LocationRemoveOutput;
use crate::{
	context::CoreContext,
	infra::action::{error::ActionError, LibraryAction},
	location::manager::LocationManager,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LocationRemoveInput {
	pub location_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRemoveAction {
	input: LocationRemoveInput,
}

impl LocationRemoveAction {
	/// Create a new location remove action
	pub fn new(input: LocationRemoveInput) -> Self {
		Self { input }
	}
}

// Implement the unified LibraryAction (replaces ActionHandler)
impl LibraryAction for LocationRemoveAction {
	type Input = LocationRemoveInput;
	type Output = LocationRemoveOutput;

	fn from_input(input: LocationRemoveInput) -> Result<Self, String> {
		Ok(LocationRemoveAction::new(input))
	}

	async fn execute(
		self,
		library: std::sync::Arc<crate::library::Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		// Remove the location from DB
		// Stop new work before the rows go: the watcher would queue indexing
		// for the tree, and live jobs keep writing while the delete waits for
		// the write lock.
		if let Some(watcher) = context.get_fs_watcher().await {
			if let Err(e) = watcher.unwatch_location(self.input.location_id).await {
				tracing::warn!(
					"Failed to unwatch location {}: {}",
					self.input.location_id,
					e
				);
			}
		}
		let cancelled = library
			.jobs()
			.cancel_location_jobs(self.input.location_id)
			.await;
		if cancelled > 0 {
			tracing::info!(
				"Cancelled {} jobs of location {}",
				cancelled,
				self.input.location_id
			);
		}

		let location_manager = LocationManager::new(context.events.as_ref().clone());
		location_manager
			.remove_location(&library, self.input.location_id)
			.await
			.map_err(|e| ActionError::Internal(e.to_string()))?;

		Ok(LocationRemoveOutput::new(self.input.location_id, None))
	}

	fn action_kind(&self) -> &'static str {
		"locations.remove"
	}
}

// Register action
crate::register_library_action!(LocationRemoveAction, "locations.remove");
