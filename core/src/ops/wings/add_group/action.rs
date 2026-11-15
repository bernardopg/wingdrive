use super::{input::AddGroupInput, output::AddGroupOutput};
use crate::{
	context::CoreContext,
	domain::WingGroup,
	infra::action::{
		error::{ActionError, ActionResult},
		LibraryAction,
	},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddGroupAction {
	input: AddGroupInput,
}

impl LibraryAction for AddGroupAction {
	type Input = AddGroupInput;
	type Output = AddGroupOutput;

	fn from_input(input: AddGroupInput) -> Result<Self, String> {
		if input.name.trim().is_empty() {
			return Err("Group name cannot be empty".to_string());
		}

		Ok(Self { input })
	}

	async fn execute(
		self,
		library: std::sync::Arc<crate::library::Library>,
		_context: std::sync::Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let db = library.db().conn();

		// Verify wing exists
		let wing_model = crate::infra::db::entities::wing::Entity::find()
			.filter(crate::infra::db::entities::wing::Column::Uuid.eq(self.input.wing_id))
			.one(db)
			.await
			.map_err(ActionError::SeaOrm)?
			.ok_or_else(|| {
				ActionError::Internal(format!("Wing {} not found", self.input.wing_id))
			})?;

		// Get max order for this wing
		let max_order = crate::infra::db::entities::wing_group::Entity::find()
			.filter(crate::infra::db::entities::wing_group::Column::WingId.eq(wing_model.id))
			.order_by_desc(crate::infra::db::entities::wing_group::Column::Order)
			.one(db)
			.await
			.map_err(ActionError::SeaOrm)?
			.map(|g| g.order)
			.unwrap_or(-1);

		let group_id = uuid::Uuid::new_v4();
		let now = Utc::now();

		// Serialize group_type to JSON
		let group_type_json = serde_json::to_string(&self.input.group_type)
			.map_err(|e| ActionError::Internal(format!("Failed to serialize group_type: {}", e)))?;

		let active_model = crate::infra::db::entities::wing_group::ActiveModel {
			id: sea_orm::NotSet,
			uuid: Set(group_id),
			wing_id: Set(wing_model.id),
			name: Set(self.input.name.clone()),
			group_type: Set(group_type_json),
			is_collapsed: Set(false),
			order: Set(max_order + 1),
			created_at: Set(now.into()),
		};

		let result = active_model.insert(db).await.map_err(ActionError::SeaOrm)?;

		// Sync to peers (emits direct event)
		library
			.sync_model_with_db(&result, crate::infra::sync::ChangeType::Insert, db)
			.await
			.map_err(|e| ActionError::Internal(format!("Failed to sync group: {}", e)))?;

		// Emit virtual resource events (wing_layout) via ResourceManager
		let resource_manager = crate::domain::ResourceManager::new(
			std::sync::Arc::new(library.db().conn().clone()),
			library.event_bus().clone(),
		);
		resource_manager
			.emit_resource_events("wing_group", vec![result.uuid])
			.await
			.map_err(|e| ActionError::Internal(format!("Failed to emit resource events: {}", e)))?;

		let group = WingGroup {
			id: result.uuid,
			wing_id: self.input.wing_id,
			name: result.name,
			group_type: self.input.group_type,
			is_collapsed: result.is_collapsed,
			order: result.order,
			created_at: result.created_at.into(),
		};

		Ok(AddGroupOutput { group })
	}

	fn action_kind(&self) -> &'static str {
		"wings.add_group"
	}

	async fn validate(
		&self,
		_library: &std::sync::Arc<crate::library::Library>,
		_context: std::sync::Arc<CoreContext>,
	) -> Result<crate::infra::action::ValidationResult, ActionError> {
		Ok(crate::infra::action::ValidationResult::Success { metadata: None })
	}
}

crate::register_library_action!(AddGroupAction, "wings.add_group");
