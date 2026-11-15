//! WingItem entity

use crate::infra::sync::Syncable;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "wing_items")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub uuid: Uuid,
	/// Sent as `space_id` (`space_uuid` after FK mapping) for devices on 3.3.1 and older.
	#[serde(rename = "space_id")]
	pub wing_id: i32,
	pub group_id: Option<i32>,    // Nullable - None = wing-level item
	pub entry_uuid: Option<Uuid>, // Nullable - populated for Path items
	pub item_type: String,        // JSON-serialized ItemType enum
	pub order: i32,
	pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::wing::Entity",
		from = "Column::WingId",
		to = "super::wing::Column::Id"
	)]
	Wing,
	#[sea_orm(
		belongs_to = "super::wing_group::Entity",
		from = "Column::GroupId",
		to = "super::wing_group::Column::Id"
	)]
	WingGroup,
}

impl Related<super::wing::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Wing.def()
	}
}

impl Related<super::wing_group::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::WingGroup.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

// Syncable Implementation
//
// WingItems sync with their parent WingGroup
impl Syncable for Model {
	// Sync model names and wire fields keep the names WingDrive used before Wings were
	// renamed (3.3.1 and older), so devices on either version keep syncing.
	const SYNC_MODEL: &'static str = "space_item";

	fn sync_id(&self) -> Uuid {
		self.uuid
	}

	fn version(&self) -> i64 {
		self.created_at.timestamp()
	}

	fn exclude_fields() -> Option<&'static [&'static str]> {
		Some(&["id"]) // Don't exclude FK fields - needed for UUID conversion
	}

	fn sync_depends_on() -> &'static [&'static str] {
		&["space", "space_group"]
	}

	fn foreign_key_mappings() -> Vec<crate::infra::sync::FKMapping> {
		vec![
			crate::infra::sync::FKMapping::new("space_id", "wings"),
			crate::infra::sync::FKMapping::new("group_id", "wing_groups"),
			// Note: entry_uuid is NOT listed here because it's already a UUID field.
			// FKMapping is only for integer ID fields that need ID→UUID conversion.
		]
	}

	// FK Lookup Methods (wing_item is FK target - rare but consistent pattern)
	async fn lookup_id_by_uuid(
		uuid: Uuid,
		db: &DatabaseConnection,
	) -> Result<Option<i32>, sea_orm::DbErr> {
		use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
		Ok(Entity::find()
			.filter(Column::Uuid.eq(uuid))
			.one(db)
			.await?
			.map(|i| i.id))
	}

	async fn lookup_uuid_by_id(
		id: i32,
		db: &DatabaseConnection,
	) -> Result<Option<Uuid>, sea_orm::DbErr> {
		Ok(Entity::find_by_id(id).one(db).await?.map(|i| i.uuid))
	}

	async fn batch_lookup_ids_by_uuids(
		uuids: std::collections::HashSet<Uuid>,
		db: &DatabaseConnection,
	) -> Result<std::collections::HashMap<Uuid, i32>, sea_orm::DbErr> {
		use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
		if uuids.is_empty() {
			return Ok(std::collections::HashMap::new());
		}
		let records = Entity::find()
			.filter(Column::Uuid.is_in(uuids))
			.all(db)
			.await?;
		Ok(records.into_iter().map(|r| (r.uuid, r.id)).collect())
	}

	async fn batch_lookup_uuids_by_ids(
		ids: std::collections::HashSet<i32>,
		db: &DatabaseConnection,
	) -> Result<std::collections::HashMap<i32, Uuid>, sea_orm::DbErr> {
		use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
		if ids.is_empty() {
			return Ok(std::collections::HashMap::new());
		}
		let records = Entity::find().filter(Column::Id.is_in(ids)).all(db).await?;
		Ok(records.into_iter().map(|r| (r.id, r.uuid)).collect())
	}

	async fn query_for_sync(
		_device_id: Option<Uuid>,
		since: Option<chrono::DateTime<chrono::Utc>>,
		cursor: Option<(chrono::DateTime<chrono::Utc>, Uuid)>,
		batch_size: usize,
		db: &DatabaseConnection,
	) -> Result<Vec<(Uuid, serde_json::Value, chrono::DateTime<chrono::Utc>)>, sea_orm::DbErr> {
		use crate::infra::sync::Syncable;
		use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

		let mut query = Entity::find();

		if let Some(since_time) = since {
			query = query.filter(Column::CreatedAt.gte(since_time));
		}

		if let Some((cursor_ts, cursor_uuid)) = cursor {
			query = query.filter(
				Condition::any().add(Column::CreatedAt.gt(cursor_ts)).add(
					Condition::all()
						.add(Column::CreatedAt.eq(cursor_ts))
						.add(Column::Uuid.gt(cursor_uuid)),
				),
			);
		}

		query = query
			.order_by_asc(Column::CreatedAt)
			.order_by_asc(Column::Uuid)
			.limit(batch_size as u64);

		let results = query.all(db).await?;

		let mut sync_results = Vec::new();
		for item in results {
			let mut json = match item.to_sync_json() {
				Ok(j) => j,
				Err(e) => {
					tracing::error!("Failed to serialize wing_item {}: {}", item.uuid, e);
					continue;
				}
			};

			// Convert FK integer IDs to UUIDs
			for fk in <Model as Syncable>::foreign_key_mappings() {
				if let Err(e) =
					crate::infra::sync::fk_mapper::convert_fk_to_uuid(&mut json, &fk, db).await
				{
					tracing::warn!(
						error = %e,
						uuid = %item.uuid,
						"Failed to convert FK to UUID for wing_item"
					);
					continue;
				}
			}

			sync_results.push((item.uuid, json, item.created_at));
		}

		Ok(sync_results)
	}

	async fn apply_shared_change(
		entry: crate::infra::sync::SharedChangeEntry,
		db: &DatabaseConnection,
	) -> Result<(), sea_orm::DbErr> {
		use crate::infra::sync::{fk_mapper, ChangeType};
		use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, QueryFilter, Set};

		match entry.change_type {
			ChangeType::Insert | ChangeType::Update => {
				// Map UUIDs to local IDs for FK fields
				let data =
					fk_mapper::map_sync_json_to_local(entry.data, Self::foreign_key_mappings(), db)
						.await
						.map_err(|e| sea_orm::DbErr::Custom(format!("FK mapping failed: {}", e)))?;

				let data = data.as_object().ok_or_else(|| {
					sea_orm::DbErr::Custom("WingItem data is not an object".to_string())
				})?;

				let uuid: Uuid = serde_json::from_value(
					data.get("uuid")
						.ok_or_else(|| sea_orm::DbErr::Custom("Missing uuid".to_string()))?
						.clone(),
				)
				.map_err(|e| sea_orm::DbErr::Custom(format!("Invalid uuid: {}", e)))?;

				let wing_id: i32 = serde_json::from_value(
					data.get("space_id")
						.ok_or_else(|| sea_orm::DbErr::Custom("Missing wing_id".to_string()))?
						.clone(),
				)
				.map_err(|e| sea_orm::DbErr::Custom(format!("Invalid wing_id: {}", e)))?;

				let group_id: Option<i32> = serde_json::from_value(
					data.get("group_id")
						.ok_or_else(|| sea_orm::DbErr::Custom("Missing group_id".to_string()))?
						.clone(),
				)
				.map_err(|e| sea_orm::DbErr::Custom(format!("Invalid group_id: {}", e)))?;

				let entry_uuid: Option<Uuid> = data
					.get("entry_uuid")
					.map(|v| serde_json::from_value(v.clone()).ok())
					.flatten();

				let active = ActiveModel {
					id: NotSet,
					uuid: Set(uuid),
					wing_id: Set(wing_id),
					group_id: Set(group_id),
					entry_uuid: Set(entry_uuid),
					item_type: Set(serde_json::from_value(
						data.get("item_type")
							.ok_or_else(|| sea_orm::DbErr::Custom("Missing item_type".to_string()))?
							.clone(),
					)
					.map_err(|e| sea_orm::DbErr::Custom(format!("Invalid item_type: {}", e)))?),
					order: Set(serde_json::from_value(
						data.get("order")
							.ok_or_else(|| sea_orm::DbErr::Custom("Missing order".to_string()))?
							.clone(),
					)
					.map_err(|e| sea_orm::DbErr::Custom(format!("Invalid order: {}", e)))?),
					created_at: Set(serde_json::from_value(
						data.get("created_at")
							.ok_or_else(|| {
								sea_orm::DbErr::Custom("Missing created_at".to_string())
							})?
							.clone(),
					)
					.map_err(|e| sea_orm::DbErr::Custom(format!("Invalid created_at: {}", e)))?),
				};

				// Atomic upsert by UUID to prevent race conditions
				Entity::insert(active)
					.on_conflict(
						sea_orm::sea_query::OnConflict::column(Column::Uuid)
							.update_columns([Column::GroupId, Column::ItemType, Column::Order])
							.to_owned(),
					)
					.exec(db)
					.await?;

				Ok(())
			}
			ChangeType::Delete => {
				let data = entry.data.as_object().ok_or_else(|| {
					sea_orm::DbErr::Custom("WingItem data is not an object".to_string())
				})?;

				let uuid: Uuid = serde_json::from_value(
					data.get("uuid")
						.ok_or_else(|| sea_orm::DbErr::Custom("Missing uuid".to_string()))?
						.clone(),
				)
				.map_err(|e| sea_orm::DbErr::Custom(format!("Invalid uuid: {}", e)))?;

				Entity::delete_many()
					.filter(Column::Uuid.eq(uuid))
					.exec(db)
					.await?;

				Ok(())
			}
		}
	}
}

// Register with sync system via inventory. The model name stays the pre-Wings one (see SYNC_MODEL).
crate::register_syncable_shared!(Model, "space_item", "wing_items");

#[cfg(test)]
mod tests {
	use super::*;
	use crate::infra::db::entities::wing_group;

	/// Devices on 3.3.1 and older read `space_id`; the record must still carry it.
	#[test]
	fn groups_and_items_name_their_wing_space_id_on_the_wire() {
		let now = chrono::Utc::now();
		let item = Model {
			id: 1,
			uuid: Uuid::nil(),
			wing_id: 7,
			group_id: Some(3),
			entry_uuid: None,
			item_type: "\"Recents\"".into(),
			order: 0,
			created_at: now,
		};
		let group = wing_group::Model {
			id: 3,
			uuid: Uuid::nil(),
			wing_id: 7,
			name: "Projects".into(),
			group_type: "\"Custom\"".into(),
			is_collapsed: false,
			order: 0,
			created_at: now,
		};
		for json in [item.to_sync_json().unwrap(), group.to_sync_json().unwrap()] {
			assert_eq!(json["space_id"], 7, "{json}");
			assert!(json.get("wing_id").is_none(), "{json}");
			assert!(json.get("id").is_none(), "{json}");
		}
		let fks = <Model as Syncable>::foreign_key_mappings();
		assert_eq!(fks[0].uuid_field_name(), "space_uuid");
		assert_eq!(Model::SYNC_MODEL, "space_item");
		assert_eq!(wing_group::Model::SYNC_MODEL, "space_group");
		assert_eq!(super::super::wing::Model::SYNC_MODEL, "space");
	}
}
