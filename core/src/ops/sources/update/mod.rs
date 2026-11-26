//! Source update action: rename a source or change its config.

use crate::{
	context::CoreContext,
	infra::action::{error::ActionError, LibraryAction},
	library::Library,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;

/// Input for changing a source.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateSourceInput {
	pub source_id: String,
	/// New display name, or none to keep it.
	pub name: Option<String>,
	/// Config fields to change, or none to keep the config. A secret field that is empty or
	/// missing keeps its stored value.
	pub config: Option<serde_json::Value>,
}

/// The source after the change.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateSourceOutput {
	pub id: String,
	pub name: String,
	pub adapter_id: String,
	pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSourceAction {
	input: UpdateSourceInput,
}

impl LibraryAction for UpdateSourceAction {
	type Input = UpdateSourceInput;
	type Output = UpdateSourceOutput;

	fn from_input(input: UpdateSourceInput) -> Result<Self, String> {
		if input.source_id.trim().is_empty() {
			return Err("source_id cannot be empty".to_string());
		}
		if input
			.name
			.as_deref()
			.is_some_and(|name| name.trim().is_empty())
		{
			return Err("Source name cannot be empty".to_string());
		}
		if input
			.config
			.as_ref()
			.is_some_and(|config| !config.is_object())
		{
			return Err("config must be an object".to_string());
		}
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		if library.source_manager().is_none() {
			library.init_source_manager().await.map_err(|e| {
				ActionError::Internal(format!("Failed to init source manager: {e}"))
			})?;
		}
		let source_manager = library
			.source_manager()
			.ok_or_else(|| ActionError::Internal("Source manager not available".to_string()))?;

		let source = source_manager
			.update_source(
				&self.input.source_id,
				self.input.name.as_deref().map(str::trim),
				self.input.config,
			)
			.await
			.map_err(ActionError::Internal)?;

		Ok(UpdateSourceOutput {
			id: source.id,
			name: source.name,
			adapter_id: source.adapter_id,
			status: source.status,
		})
	}

	fn action_kind(&self) -> &'static str {
		"sources.update"
	}
}

crate::register_library_action!(UpdateSourceAction, "sources.update");
