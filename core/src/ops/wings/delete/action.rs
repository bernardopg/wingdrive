use super::{input::WingDeleteInput, output::WingDeleteOutput};
use crate::{
	context::CoreContext,
	infra::action::{
		error::{ActionError, ActionResult},
		LibraryAction,
	},
};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WingDeleteAction {
	input: WingDeleteInput,
}

impl LibraryAction for WingDeleteAction {
	type Input = WingDeleteInput;
	type Output = WingDeleteOutput;

	fn from_input(input: WingDeleteInput) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: std::sync::Arc<crate::library::Library>,
		_context: std::sync::Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let db = library.db().conn();

		let wing_model = crate::infra::db::entities::wing::Entity::find()
			.filter(crate::infra::db::entities::wing::Column::Uuid.eq(self.input.wing_id))
			.one(db)
			.await
			.map_err(ActionError::SeaOrm)?
			.ok_or_else(|| {
				ActionError::Internal(format!("Wing {} not found", self.input.wing_id))
			})?;

		let wing_id = wing_model.uuid;

		// Groups and items go with the wing through the foreign keys, here and
		// on every device that applies the synced deletion.
		// Record the deletion for other devices first: if that fails nothing is
		// deleted and the action can be retried, instead of a local deletion
		// that no device ever hears about.
		library
			.sync_model(&wing_model, crate::infra::sync::ChangeType::Delete)
			.await
			.map_err(|e| ActionError::Internal(format!("Failed to sync wing: {}", e)))?;
		wing_model.delete(db).await.map_err(ActionError::SeaOrm)?;

		// Emit ResourceDeleted event for real-time UI updates using EventEmitter
		use crate::domain::{resource::EventEmitter, Wing};
		Wing::emit_deleted(wing_id, library.event_bus());

		Ok(WingDeleteOutput { success: true })
	}

	fn action_kind(&self) -> &'static str {
		"wings.delete"
	}

	async fn validate(
		&self,
		_library: &std::sync::Arc<crate::library::Library>,
		_context: std::sync::Arc<CoreContext>,
	) -> Result<crate::infra::action::ValidationResult, ActionError> {
		Ok(crate::infra::action::ValidationResult::Success { metadata: None })
	}
}

crate::register_library_action!(WingDeleteAction, "wings.delete");
