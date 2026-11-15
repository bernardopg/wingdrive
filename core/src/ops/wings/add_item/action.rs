use super::{input::AddItemInput, output::AddItemOutput};
use crate::{
	context::CoreContext,
	domain::{addressing::WingPath, ItemType, WingItem},
	infra::action::{
		error::{ActionError, ActionResult},
		LibraryAction,
	},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddItemAction {
	input: AddItemInput,
}

impl LibraryAction for AddItemAction {
	type Input = AddItemInput;
	type Output = AddItemOutput;

	fn from_input(input: AddItemInput) -> Result<Self, String> {
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

		// Verify the group exists in this wing if group_id is provided
		let group_model_id = if let Some(group_id) = self.input.group_id {
			let group_model = crate::infra::db::entities::wing_group::Entity::find()
				.filter(crate::infra::db::entities::wing_group::Column::Uuid.eq(group_id))
				.filter(crate::infra::db::entities::wing_group::Column::WingId.eq(wing_model.id))
				.one(db)
				.await
				.map_err(ActionError::SeaOrm)?
				.ok_or_else(|| ActionError::Internal(format!("Group {} not found", group_id)))?;

			Some(group_model.id)
		} else {
			None
		};

		// Get max order (either for wing-level or group-level items)
		let max_order = if let Some(group_id) = group_model_id {
			// Max order within group
			crate::infra::db::entities::wing_item::Entity::find()
				.filter(crate::infra::db::entities::wing_item::Column::GroupId.eq(Some(group_id)))
				.order_by_desc(crate::infra::db::entities::wing_item::Column::Order)
				.one(db)
				.await
				.map_err(ActionError::SeaOrm)?
				.map(|i| i.order)
				.unwrap_or(-1)
		} else {
			// Max order for wing-level items
			crate::infra::db::entities::wing_item::Entity::find()
				.filter(crate::infra::db::entities::wing_item::Column::WingId.eq(wing_model.id))
				.filter(crate::infra::db::entities::wing_item::Column::GroupId.is_null())
				.order_by_desc(crate::infra::db::entities::wing_item::Column::Order)
				.one(db)
				.await
				.map_err(ActionError::SeaOrm)?
				.map(|i| i.order)
				.unwrap_or(-1)
		};

		let item_id = uuid::Uuid::new_v4();
		let now = Utc::now();

		// Resolve entry_uuid if this is a Path item
		let entry_uuid = if let ItemType::Path { ref wing_path } = self.input.item_type {
			tracing::info!("Resolving WingPath to entry_uuid: {:?}", wing_path);
			let resolved = resolve_wing_path_to_entry_uuid(wing_path, db).await;
			tracing::info!("Resolved entry_uuid: {:?}", resolved);
			resolved
		} else {
			None
		};

		// Serialize item_type to JSON
		let item_type_json = serde_json::to_string(&self.input.item_type)
			.map_err(|e| ActionError::Internal(format!("Failed to serialize item_type: {}", e)))?;

		let active_model = crate::infra::db::entities::wing_item::ActiveModel {
			id: NotSet,
			uuid: Set(item_id),
			wing_id: Set(wing_model.id),
			group_id: Set(group_model_id),
			entry_uuid: Set(entry_uuid),
			item_type: Set(item_type_json),
			order: Set(max_order + 1),
			created_at: Set(now.into()),
		};

		let result = active_model.insert(db).await.map_err(ActionError::SeaOrm)?;

		// Sync to peers (emits direct event)
		library
			.sync_model_with_db(&result, crate::infra::sync::ChangeType::Insert, db)
			.await
			.map_err(|e| ActionError::Internal(format!("Failed to sync item: {}", e)))?;

		// Emit virtual resource events (wing_layout) via ResourceManager
		let resource_manager = crate::domain::ResourceManager::new(
			std::sync::Arc::new(library.db().conn().clone()),
			library.event_bus().clone(),
		);
		resource_manager
			.emit_resource_events("wing_item", vec![result.uuid])
			.await
			.map_err(|e| ActionError::Internal(format!("Failed to emit resource events: {}", e)))?;

		let item = WingItem {
			id: result.uuid,
			wing_id: self.input.wing_id,
			group_id: self.input.group_id,
			item_type: self.input.item_type,
			order: result.order,
			created_at: result.created_at.into(),
			resolved_file: None, // Not populated in add_item, only in get_layout
		};

		Ok(AddItemOutput { item })
	}

	fn action_kind(&self) -> &'static str {
		"wings.add_item"
	}

	async fn validate(
		&self,
		_library: &std::sync::Arc<crate::library::Library>,
		_context: std::sync::Arc<CoreContext>,
	) -> Result<crate::infra::action::ValidationResult, ActionError> {
		Ok(crate::infra::action::ValidationResult::Success { metadata: None })
	}
}

crate::register_library_action!(AddItemAction, "wings.add_item");

/// Resolve a WingPath to the UUID of its indexed entry.
///
/// Matching by name and extension found a file `notes.txt` for a folder
/// `notes` pinned beside it, and never found a folder with a dot in its name,
/// so this uses the path resolver the file queries use.
async fn resolve_wing_path_to_entry_uuid(
	wing_path: &WingPath,
	db: &sea_orm::DatabaseConnection,
) -> Option<uuid::Uuid> {
	match crate::ops::indexing::PathResolver::resolve_to_entry(db, wing_path).await {
		Ok(entry) => entry.and_then(|entry| entry.uuid),
		Err(error) => {
			tracing::warn!(
				"Failed to resolve {} to an entry: {}",
				wing_path.display(),
				error
			);
			None
		}
	}
}
