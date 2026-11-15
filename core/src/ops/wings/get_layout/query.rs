use super::output::WingLayoutOutput;
use crate::domain::{
	addressing::WingPath, ContentKind, File, GroupType, ItemType, Wing, WingGroup,
	WingGroupWithItems, WingItem, WingLayout,
};
use crate::infra::db::entities::{content_identity, entry, sidecar, wing_item};
use crate::infra::query::{QueryError, QueryResult};
use crate::{context::CoreContext, infra::query::LibraryQuery};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingLayoutQueryInput {
	pub wing_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingLayoutQuery {
	wing_id: Uuid,
}

impl LibraryQuery for WingLayoutQuery {
	type Input = WingLayoutQueryInput;
	type Output = WingLayoutOutput;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self {
			wing_id: input.wing_id,
		})
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let library_id = session
			.current_library_id
			.ok_or_else(|| QueryError::Internal("No library selected".to_string()))?;

		let library = context
			.libraries()
			.await
			.get_library(library_id)
			.await
			.ok_or_else(|| QueryError::Internal("Library not found".to_string()))?;

		let db = library.db().conn();

		// Get wing
		let wing_model = crate::infra::db::entities::wing::Entity::find()
			.filter(crate::infra::db::entities::wing::Column::Uuid.eq(self.wing_id))
			.one(db)
			.await?
			.ok_or_else(|| QueryError::Internal(format!("Wing {} not found", self.wing_id)))?;

		let wing = Wing {
			id: wing_model.uuid,
			name: wing_model.name,
			icon: wing_model.icon,
			color: wing_model.color,
			order: wing_model.order,
			created_at: wing_model.created_at.into(),
			updated_at: wing_model.updated_at.into(),
		};

		// Get wing-level items (no group)
		let wing_item_models = crate::infra::db::entities::wing_item::Entity::find()
			.filter(crate::infra::db::entities::wing_item::Column::WingId.eq(wing_model.id))
			.filter(crate::infra::db::entities::wing_item::Column::GroupId.is_null())
			.order_by_asc(crate::infra::db::entities::wing_item::Column::Order)
			.all(db)
			.await?;

		let mut wing_items = Vec::new();

		for item_model in wing_item_models {
			let item_type: ItemType = serde_json::from_str(&item_model.item_type)
				.map_err(|e| QueryError::Internal(format!("Failed to parse item_type: {}", e)))?;

			// Resolve entry if entry_uuid is set
			let resolved_file = if let Some(entry_uuid) = item_model.entry_uuid {
				tracing::debug!(
					"Wing item {} has entry_uuid: {}",
					item_model.uuid,
					entry_uuid
				);
				if let Ok(Some(entry_model)) = entry::Entity::find()
					.filter(entry::Column::Uuid.eq(entry_uuid))
					.one(db)
					.await
				{
					let file = build_file_from_entry(entry_model, &item_type, db)
						.await
						.map(Box::new);
					file
				} else {
					tracing::warn!("Entry {} not found for wing item", entry_uuid);
					None
				}
			} else {
				tracing::debug!("Wing item {} has no entry_uuid", item_model.uuid);
				None
			};

			wing_items.push(WingItem {
				id: item_model.uuid,
				wing_id: self.wing_id,
				group_id: None,
				item_type,
				order: item_model.order,
				created_at: item_model.created_at.into(),
				resolved_file,
			});
		}

		// Get groups for this wing
		let group_models = crate::infra::db::entities::wing_group::Entity::find()
			.filter(crate::infra::db::entities::wing_group::Column::WingId.eq(wing_model.id))
			.order_by_asc(crate::infra::db::entities::wing_group::Column::Order)
			.all(db)
			.await?;

		let mut groups = Vec::new();

		for group_model in group_models {
			// Parse group_type JSON
			let group_type: GroupType = serde_json::from_str(&group_model.group_type)
				.map_err(|e| QueryError::Internal(format!("Failed to parse group_type: {}", e)))?;

			let group = WingGroup {
				id: group_model.uuid,
				wing_id: self.wing_id,
				name: group_model.name,
				group_type,
				is_collapsed: group_model.is_collapsed,
				order: group_model.order,
				created_at: group_model.created_at.into(),
			};

			// Get items for this group
			let item_models = crate::infra::db::entities::wing_item::Entity::find()
				.filter(
					crate::infra::db::entities::wing_item::Column::GroupId.eq(Some(group_model.id)),
				)
				.order_by_asc(crate::infra::db::entities::wing_item::Column::Order)
				.all(db)
				.await?;

			let mut items = Vec::new();

			for item_model in item_models {
				// Parse item_type JSON
				let item_type: ItemType =
					serde_json::from_str(&item_model.item_type).map_err(|e| {
						QueryError::Internal(format!("Failed to parse item_type: {}", e))
					})?;

				// Resolve entry if entry_uuid is set
				let resolved_file = if let Some(entry_uuid) = item_model.entry_uuid {
					tracing::debug!(
						"Group item {} has entry_uuid: {}",
						item_model.uuid,
						entry_uuid
					);
					if let Ok(Some(entry_model)) = entry::Entity::find()
						.filter(entry::Column::Uuid.eq(entry_uuid))
						.one(db)
						.await
					{
						tracing::debug!("Found entry: name={}", entry_model.name);
						let file = build_file_from_entry(entry_model, &item_type, db)
							.await
							.map(Box::new);
						tracing::info!(
							"Built file for group item: {:?}",
							file.as_ref().map(|f| &f.name)
						);
						file
					} else {
						tracing::warn!("Entry {} not found for group item", entry_uuid);
						None
					}
				} else {
					None
				};

				items.push(WingItem {
					id: item_model.uuid,
					wing_id: self.wing_id,
					group_id: Some(group_model.uuid),
					item_type,
					order: item_model.order,
					created_at: item_model.created_at.into(),
					resolved_file,
				});
			}

			groups.push(WingGroupWithItems { group, items });
		}

		let layout = WingLayout {
			id: self.wing_id,
			wing,
			wing_items,
			groups,
		};

		Ok(layout)
	}
}

crate::register_library_query!(WingLayoutQuery, "wings.get_layout");

/// Build a minimal File object from an entry model (for sidebar display)
async fn build_file_from_entry(
	entry_model: entry::Model,
	item_type: &ItemType,
	db: &DatabaseConnection,
) -> Option<File> {
	// Get the WingPath from item_type
	let wing_path = match item_type {
		ItemType::Path { wing_path } => wing_path.clone(),
		_ => return None,
	};

	// Get content identity if available
	let content_identity = if let Some(content_id) = entry_model.content_id {
		content_identity::Entity::find_by_id(content_id)
			.one(db)
			.await
			.ok()
			.flatten()
			.map(|ci| crate::domain::ContentIdentity {
				uuid: ci.uuid.unwrap_or_else(Uuid::new_v4),
				kind: ContentKind::from_id(ci.kind_id),
				content_hash: ci.content_hash,
				integrity_hash: ci.integrity_hash,
				mime_type_id: ci.mime_type_id,
				text_content: ci.text_content,
				total_size: ci.total_size,
				entry_count: ci.entry_count,
				first_seen_at: ci.first_seen_at,
				last_verified_at: ci.last_verified_at,
			})
	} else {
		None
	};

	// Get sidecars for thumbnails
	let sidecars = if let Some(ref ci) = content_identity {
		if let Some(uuid) = Some(ci.uuid) {
			sidecar::Entity::find()
				.filter(sidecar::Column::ContentUuid.eq(uuid))
				.all(db)
				.await
				.ok()
				.unwrap_or_default()
				.into_iter()
				.map(|s| crate::domain::Sidecar {
					id: s.id,
					content_uuid: s.content_uuid,
					kind: s.kind,
					variant: s.variant,
					format: s.format,
					status: s.status,
					size: s.size,
					created_at: s.created_at,
					updated_at: s.updated_at,
				})
				.collect()
		} else {
			Vec::new()
		}
	} else {
		Vec::new()
	};

	let mut file = File::from_entity_model(entry_model, wing_path);
	file.content_identity = content_identity;
	file.sidecars = sidecars;
	if let Some(ref ci) = file.content_identity {
		file.content_kind = ci.kind;
	}

	Some(file)
}
