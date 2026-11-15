use super::{
	input::{ReorderGroupsInput, ReorderItemsInput},
	output::ReorderOutput,
};
use crate::{
	context::CoreContext,
	infra::{
		action::{
			error::{ActionError, ActionResult},
			LibraryAction,
		},
		db::entities::{wing, wing_group, wing_item},
	},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderGroupsAction {
	input: ReorderGroupsInput,
}

impl LibraryAction for ReorderGroupsAction {
	type Input = ReorderGroupsInput;
	type Output = ReorderOutput;

	fn from_input(input: ReorderGroupsInput) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: std::sync::Arc<crate::library::Library>,
		_context: std::sync::Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let db = library.db().conn();

		let wing_model = wing::Entity::find()
			.filter(wing::Column::Uuid.eq(self.input.wing_id))
			.one(db)
			.await
			.map_err(ActionError::SeaOrm)?
			.ok_or_else(|| {
				ActionError::Internal(format!("Wing {} not found", self.input.wing_id))
			})?;

		// Every group must belong to this wing before any order changes
		let mut groups = Vec::with_capacity(self.input.group_ids.len());
		for group_id in &self.input.group_ids {
			groups.push(
				wing_group::Entity::find()
					.filter(wing_group::Column::Uuid.eq(*group_id))
					.filter(wing_group::Column::WingId.eq(wing_model.id))
					.one(db)
					.await
					.map_err(ActionError::SeaOrm)?
					.ok_or_else(|| {
						ActionError::Internal(format!("Group {} not found", group_id))
					})?,
			);
		}

		let txn = db.begin().await.map_err(ActionError::SeaOrm)?;
		let mut updated = Vec::with_capacity(groups.len());
		for (index, group) in groups.into_iter().enumerate() {
			let mut active_model: wing_group::ActiveModel = group.into();
			active_model.order = Set(index as i32);
			updated.push(
				active_model
					.update(&txn)
					.await
					.map_err(ActionError::SeaOrm)?,
			);
		}
		txn.commit().await.map_err(ActionError::SeaOrm)?;

		// `wing_id` goes out as the wing UUID; peers number rows differently
		for group in &updated {
			library
				.sync_model_with_db(group, crate::infra::sync::ChangeType::Update, db)
				.await
				.map_err(|e| ActionError::Internal(format!("Failed to sync wing group: {}", e)))?;
		}

		emit_layout_events(&library, "wing_group", self.input.group_ids).await?;

		Ok(ReorderOutput { success: true })
	}

	fn action_kind(&self) -> &'static str {
		"wings.reorder_groups"
	}

	async fn validate(
		&self,
		_library: &std::sync::Arc<crate::library::Library>,
		_context: std::sync::Arc<CoreContext>,
	) -> Result<crate::infra::action::ValidationResult, ActionError> {
		Ok(crate::infra::action::ValidationResult::Success { metadata: None })
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderItemsAction {
	input: ReorderItemsInput,
}

impl LibraryAction for ReorderItemsAction {
	type Input = ReorderItemsInput;
	type Output = ReorderOutput;

	fn from_input(input: ReorderItemsInput) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: std::sync::Arc<crate::library::Library>,
		_context: std::sync::Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let db = library.db().conn();

		// The group the items belong to; `None` means wing-level items
		let group_model_id = match self.input.group_id {
			Some(group_id) => Some(
				wing_group::Entity::find()
					.filter(wing_group::Column::Uuid.eq(group_id))
					.one(db)
					.await
					.map_err(ActionError::SeaOrm)?
					.ok_or_else(|| ActionError::Internal(format!("Group {} not found", group_id)))?
					.id,
			),
			None => None,
		};

		// Every item must sit in that group, or at the level of a single wing,
		// before any order changes
		let mut items: Vec<wing_item::Model> = Vec::with_capacity(self.input.item_ids.len());
		for item_id in &self.input.item_ids {
			let item = wing_item::Entity::find()
				.filter(wing_item::Column::Uuid.eq(*item_id))
				.one(db)
				.await
				.map_err(ActionError::SeaOrm)?
				.filter(|item| item.group_id == group_model_id)
				.filter(|item| {
					items
						.first()
						.is_none_or(|first| first.wing_id == item.wing_id)
				})
				.ok_or_else(|| ActionError::Internal(format!("Item {} not found", item_id)))?;
			items.push(item);
		}

		let txn = db.begin().await.map_err(ActionError::SeaOrm)?;
		let mut updated = Vec::with_capacity(items.len());
		for (index, item) in items.into_iter().enumerate() {
			let mut active_model: wing_item::ActiveModel = item.into();
			active_model.order = Set(index as i32);
			updated.push(
				active_model
					.update(&txn)
					.await
					.map_err(ActionError::SeaOrm)?,
			);
		}
		txn.commit().await.map_err(ActionError::SeaOrm)?;

		// `wing_id` and `group_id` go out as UUIDs; peers number rows differently
		for item in &updated {
			library
				.sync_model_with_db(item, crate::infra::sync::ChangeType::Update, db)
				.await
				.map_err(|e| ActionError::Internal(format!("Failed to sync wing item: {}", e)))?;
		}

		emit_layout_events(&library, "wing_item", self.input.item_ids).await?;

		Ok(ReorderOutput { success: true })
	}

	fn action_kind(&self) -> &'static str {
		"wings.reorder_items"
	}

	async fn validate(
		&self,
		_library: &std::sync::Arc<crate::library::Library>,
		_context: std::sync::Arc<CoreContext>,
	) -> Result<crate::infra::action::ValidationResult, ActionError> {
		Ok(crate::infra::action::ValidationResult::Success { metadata: None })
	}
}

crate::register_library_action!(ReorderGroupsAction, "wings.reorder_groups");
crate::register_library_action!(ReorderItemsAction, "wings.reorder_items");

/// Emits the resource events that refresh the wing layout of open windows.
async fn emit_layout_events(
	library: &crate::library::Library,
	resource_type: &str,
	ids: Vec<Uuid>,
) -> ActionResult<()> {
	crate::domain::ResourceManager::new(
		std::sync::Arc::new(library.db().conn().clone()),
		library.event_bus().clone(),
	)
	.emit_resource_events(resource_type, ids)
	.await
	.map_err(|e| ActionError::Internal(format!("Failed to emit resource events: {}", e)))
}
