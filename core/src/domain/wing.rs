//! Wings - Arc-browser-inspired sidebar organization system
//!
//! Wings allow users to create custom sidebar layouts with device-aware groups,
//! sortable items, and context-based filtering. Each Wing defines how the
//! sidebar is organized and what items are visible.

use crate::domain::addressing::WingPath;
use crate::domain::resource::Identifiable;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

/// The resource type for a sync model name.
///
/// Wings, their groups and their items still sync under the model names WingDrive used before
/// they were renamed (`space`, `space_group`, `space_item`), so devices on 3.3.1 and older keep
/// syncing. Every other model name is already its resource type.
pub fn resource_type_for_sync_model(model: &str) -> &str {
	match model {
		"space" => "wing",
		"space_group" => "wing_group",
		"space_item" => "wing_item",
		other => other,
	}
}

/// A Wing defines a sidebar layout and filtering context
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Wing {
	/// Unique identifier
	pub id: Uuid,

	/// Human-friendly name (e.g., "All Devices", "Work Files")
	pub name: String,

	/// Icon identifier (Phosphor icon name or emoji)
	pub icon: String,

	/// Color for visual identification (hex format: #RRGGBB)
	pub color: String,

	/// Sort order in wing switcher
	pub order: i32,

	/// Timestamps
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

impl Wing {
	/// Create a new wing
	pub fn new(name: String, icon: String, color: String) -> Self {
		let now = Utc::now();
		Self {
			id: Uuid::new_v4(),
			name,
			icon,
			color,
			order: 0,
			created_at: now,
			updated_at: now,
		}
	}

	/// Validate color format
	pub fn validate_color(color: &str) -> bool {
		color.len() == 7
			&& color.starts_with('#')
			&& color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
	}

	/// Create a default "All Devices" wing
	pub fn create_default() -> Self {
		Self::new(
			"All Devices".to_string(),
			"Planet".to_string(),
			"#3B82F6".to_string(),
		)
	}
}

impl Identifiable for Wing {
	fn id(&self) -> Uuid {
		self.id
	}

	fn resource_type() -> &'static str {
		"wing"
	}

	async fn from_ids(
		db: &sea_orm::DatabaseConnection,
		ids: &[Uuid],
	) -> crate::common::errors::Result<Vec<Self>>
	where
		Self: Sized,
	{
		use crate::infra::db::entities::wing;
		use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

		let wing_models = wing::Entity::find()
			.filter(wing::Column::Uuid.is_in(ids.to_vec()))
			.all(db)
			.await?;

		Ok(wing_models
			.into_iter()
			.map(|s| Wing {
				id: s.uuid,
				name: s.name,
				icon: s.icon,
				color: s.color,
				order: s.order,
				created_at: s.created_at.into(),
				updated_at: s.updated_at.into(),
			})
			.collect())
	}
}

/// A WingGroup is a collapsible section in the sidebar
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingGroup {
	/// Unique identifier
	pub id: Uuid,

	/// Wing this group belongs to
	pub wing_id: Uuid,

	/// Group name (e.g., "Quick Access", "MacBook Pro")
	pub name: String,

	/// Type of group (determines content and behavior)
	pub group_type: GroupType,

	/// Whether group is collapsed
	pub is_collapsed: bool,

	/// Sort order within wing
	pub order: i32,

	/// Timestamp
	pub created_at: DateTime<Utc>,
}

impl WingGroup {
	/// Create a new group
	pub fn new(wing_id: Uuid, name: String, group_type: GroupType) -> Self {
		Self {
			id: Uuid::new_v4(),
			wing_id,
			name,
			group_type,
			is_collapsed: false,
			order: 0,
			created_at: Utc::now(),
		}
	}

	/// Create a Quick Access group
	pub fn create_quick_access(wing_id: Uuid) -> Self {
		Self::new(wing_id, "Quick Access".to_string(), GroupType::QuickAccess)
	}

	/// Create a Device group
	pub fn create_device(wing_id: Uuid, device_id: Uuid, device_name: String) -> Self {
		Self::new(wing_id, device_name, GroupType::Device { device_id })
	}
}

impl Identifiable for WingGroup {
	fn id(&self) -> Uuid {
		self.id
	}

	fn resource_type() -> &'static str {
		"wing_group"
	}

	async fn from_ids(
		db: &sea_orm::DatabaseConnection,
		ids: &[Uuid],
	) -> crate::common::errors::Result<Vec<Self>>
	where
		Self: Sized,
	{
		use crate::infra::db::entities::{wing, wing_group};
		use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

		let group_models = wing_group::Entity::find()
			.filter(wing_group::Column::Uuid.is_in(ids.to_vec()))
			.all(db)
			.await?;

		let mut results = Vec::new();

		for group_model in group_models {
			// Fetch parent wing to get wing_id (UUID)
			let wing_model = wing::Entity::find_by_id(group_model.wing_id)
				.one(db)
				.await?;

			let wing_id = wing_model.map(|s| s.uuid).unwrap_or(group_model.uuid);

			let group_type: GroupType =
				serde_json::from_str(&group_model.group_type).map_err(|e| {
					crate::common::errors::CoreError::Other(anyhow::anyhow!(
						"Failed to parse group_type: {}",
						e
					))
				})?;

			results.push(WingGroup {
				id: group_model.uuid,
				wing_id,
				name: group_model.name,
				group_type,
				is_collapsed: group_model.is_collapsed,
				order: group_model.order,
				created_at: group_model.created_at.into(),
			});
		}

		Ok(results)
	}
}

/// Types of groups that can appear in a wing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub enum GroupType {
	/// Fixed quick navigation (Overview, Recents, Favorites)
	QuickAccess,

	/// Device with its volumes and locations as children
	Device { device_id: Uuid },

	/// All devices (library and paired) across the system
	Devices,

	/// All locations across all devices
	Locations,

	/// All volumes across all devices
	Volumes,

	/// Tag collection
	Tags,

	/// Archive data sources (email, notes, bookmarks, etc.)
	Sources,

	/// Cloud storage providers
	Cloud,

	/// User-defined custom group
	Custom,
}

/// An item within a wing (can be wing-level or within a group)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingItem {
	/// Unique identifier
	pub id: Uuid,

	/// Wing this item belongs to
	pub wing_id: Uuid,

	/// Group this item belongs to (None = wing-level item)
	pub group_id: Option<Uuid>,

	/// Type discriminant (for quick type checking)
	pub item_type: ItemType,

	/// Sort order within wing or group
	pub order: i32,

	/// Timestamp
	pub created_at: DateTime<Utc>,

	/// Resolved file data for Path items (populated by get_layout query)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub resolved_file: Option<Box<super::File>>,
}

impl WingItem {
	/// Create a new item within a group
	pub fn new(wing_id: Uuid, group_id: Uuid, item_type: ItemType) -> Self {
		Self {
			id: Uuid::new_v4(),
			wing_id,
			group_id: Some(group_id),
			item_type,
			order: 0,
			created_at: Utc::now(),
			resolved_file: None,
		}
	}

	/// Create a new wing-level item (not in any group)
	pub fn new_wing_level(wing_id: Uuid, item_type: ItemType) -> Self {
		Self {
			id: Uuid::new_v4(),
			wing_id,
			group_id: None,
			item_type,
			order: 0,
			created_at: Utc::now(),
			resolved_file: None,
		}
	}

	/// Create an Overview item
	pub fn create_overview(wing_id: Uuid, group_id: Uuid) -> Self {
		Self::new(wing_id, group_id, ItemType::Overview)
	}

	/// Create a Recents item
	pub fn create_recents(wing_id: Uuid, group_id: Uuid) -> Self {
		Self::new(wing_id, group_id, ItemType::Recents)
	}

	/// Create a Favorites item
	pub fn create_favorites(wing_id: Uuid, group_id: Uuid) -> Self {
		Self::new(wing_id, group_id, ItemType::Favorites)
	}

	/// Create a Location item
	pub fn create_location(wing_id: Uuid, group_id: Uuid, location_id: Uuid) -> Self {
		Self::new(wing_id, group_id, ItemType::Location { location_id })
	}

	/// Create a Path item (arbitrary WingPath)
	pub fn create_path(wing_id: Uuid, group_id: Uuid, wing_path: WingPath) -> Self {
		Self::new(wing_id, group_id, ItemType::Path { wing_path })
	}

	/// Create a wing-level Path item (pinned shortcut)
	pub fn create_wing_level_path(wing_id: Uuid, wing_path: WingPath) -> Self {
		Self::new_wing_level(wing_id, ItemType::Path { wing_path })
	}
}

impl Identifiable for WingItem {
	fn id(&self) -> Uuid {
		self.id
	}

	fn resource_type() -> &'static str {
		"wing_item"
	}

	fn no_merge_fields() -> &'static [&'static str] {
		&["resolved_file"]
	}

	async fn from_ids(
		db: &sea_orm::DatabaseConnection,
		ids: &[Uuid],
	) -> crate::common::errors::Result<Vec<Self>>
	where
		Self: Sized,
	{
		use crate::infra::db::entities::{entry, wing, wing_group, wing_item};
		use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

		let item_models = wing_item::Entity::find()
			.filter(wing_item::Column::Uuid.is_in(ids.to_vec()))
			.all(db)
			.await?;

		let mut results = Vec::new();

		for item_model in item_models {
			// Fetch parent wing to get wing_id (UUID)
			let wing_model = wing::Entity::find_by_id(item_model.wing_id).one(db).await?;

			let wing_id = wing_model.map(|s| s.uuid).unwrap_or(item_model.uuid);

			let item_type: ItemType = serde_json::from_str(&item_model.item_type).map_err(|e| {
				crate::common::errors::CoreError::Other(anyhow::anyhow!(
					"Failed to parse item_type: {}",
					e
				))
			})?;

			// Look up group UUID from group_id if present
			let group_id = if let Some(gid) = item_model.group_id {
				wing_group::Entity::find_by_id(gid)
					.one(db)
					.await?
					.map(|g| g.uuid)
			} else {
				None
			};

			// Build resolved_file if entry_uuid exists
			let resolved_file = if let Some(entry_uuid) = item_model.entry_uuid {
				let entry_model = entry::Entity::find()
					.filter(entry::Column::Uuid.eq(entry_uuid))
					.one(db)
					.await?;

				if let Some(entry_model) = entry_model {
					super::file::File::from_entry_model_with_item_type(entry_model, &item_type, db)
						.await
						.map(Box::new)
				} else {
					None
				}
			} else {
				None
			};

			results.push(WingItem {
				id: item_model.uuid,
				wing_id,
				group_id,
				item_type,
				order: item_model.order,
				created_at: item_model.created_at.into(),
				resolved_file,
			});
		}

		Ok(results)
	}
}

/// Types of items that can appear in a group
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub enum ItemType {
	/// Overview screen (fixed)
	Overview,

	/// Recent files (fixed)
	Recents,

	/// Favorited files (fixed)
	Favorites,

	/// File kinds (images, videos, audio, etc.)
	FileKinds,

	/// Indexed location
	Location { location_id: Uuid },

	/// Storage volume (with locations as children)
	Volume { volume_id: Uuid },

	/// Tag filter
	Tag { tag_id: Uuid },

	/// Any arbitrary path (dragged from explorer)
	Path { wing_path: WingPath },

	/// All archive data sources screen
	Sources,

	/// Specific archive data source
	Source { source_id: String },

	/// Redundancy awareness dashboard
	Redundancy,
}
/// Complete sidebar layout for a wing
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingLayout {
	/// Unique identifier (same as wing.id for cache matching)
	pub id: Uuid,

	/// The wing
	pub wing: Wing,

	/// Wing-level items (pinned shortcuts, no group)
	pub wing_items: Vec<WingItem>,

	/// Groups with their items
	pub groups: Vec<WingGroupWithItems>,
}

impl WingLayout {
	/// Construct WingLayout from wing IDs (for resource manager)
	pub async fn from_wing_ids(
		db: &sea_orm::DatabaseConnection,
		wing_ids: &[Uuid],
	) -> crate::common::errors::Result<Vec<Self>> {
		use crate::infra::db::entities::{wing, wing_group, wing_item};
		use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

		let mut layouts = Vec::new();

		for &wing_id in wing_ids {
			// Get wing
			let wing_model = wing::Entity::find()
				.filter(wing::Column::Uuid.eq(wing_id))
				.one(db)
				.await?;

			let Some(wing_model) = wing_model else {
				continue;
			};

			let wing = Wing {
				id: wing_model.uuid,
				name: wing_model.name,
				icon: wing_model.icon,
				color: wing_model.color,
				order: wing_model.order,
				created_at: wing_model.created_at.into(),
				updated_at: wing_model.updated_at.into(),
			};

			// Get wing-level items
			let wing_item_models = wing_item::Entity::find()
				.filter(wing_item::Column::WingId.eq(wing_model.id))
				.filter(wing_item::Column::GroupId.is_null())
				.order_by_asc(wing_item::Column::Order)
				.all(db)
				.await?;

			let mut wing_items = Vec::new();
			for item_model in wing_item_models {
				let item_type: ItemType =
					serde_json::from_str(&item_model.item_type).map_err(|e| {
						crate::common::errors::CoreError::Other(anyhow::anyhow!(
							"Failed to parse item_type: {}",
							e
						))
					})?;

				wing_items.push(WingItem {
					id: item_model.uuid,
					wing_id,
					group_id: None,
					item_type,
					order: item_model.order,
					created_at: item_model.created_at.into(),
					resolved_file: None,
				});
			}

			// Get groups with items
			let group_models = wing_group::Entity::find()
				.filter(wing_group::Column::WingId.eq(wing_model.id))
				.order_by_asc(wing_group::Column::Order)
				.all(db)
				.await?;

			let mut groups = Vec::new();
			for group_model in group_models {
				let group_type: GroupType =
					serde_json::from_str(&group_model.group_type).map_err(|e| {
						crate::common::errors::CoreError::Other(anyhow::anyhow!(
							"Failed to parse group_type: {}",
							e
						))
					})?;

				let group = WingGroup {
					id: group_model.uuid,
					wing_id,
					name: group_model.name,
					group_type,
					is_collapsed: group_model.is_collapsed,
					order: group_model.order,
					created_at: group_model.created_at.into(),
				};

				// Get items for this group
				let item_models = wing_item::Entity::find()
					.filter(wing_item::Column::GroupId.eq(Some(group_model.id)))
					.order_by_asc(wing_item::Column::Order)
					.all(db)
					.await?;

				let mut items = Vec::new();
				for item_model in item_models {
					let item_type: ItemType =
						serde_json::from_str(&item_model.item_type).map_err(|e| {
							crate::common::errors::CoreError::Other(anyhow::anyhow!(
								"Failed to parse item_type: {}",
								e
							))
						})?;

					items.push(WingItem {
						id: item_model.uuid,
						wing_id,
						group_id: Some(group_model.uuid),
						item_type,
						order: item_model.order,
						created_at: item_model.created_at.into(),
						resolved_file: None,
					});
				}

				groups.push(WingGroupWithItems { group, items });
			}

			layouts.push(WingLayout {
				id: wing_id,
				wing,
				wing_items,
				groups,
			});
		}

		Ok(layouts)
	}
}

impl Identifiable for WingLayout {
	fn id(&self) -> Uuid {
		self.id
	}

	fn resource_type() -> &'static str {
		"wing_layout"
	}

	fn sync_dependencies() -> &'static [&'static str] {
		&["wing", "wing_group", "wing_item"]
	}

	async fn route_from_dependency(
		db: &sea_orm::DatabaseConnection,
		dependency_type: &str,
		dependency_id: Uuid,
	) -> crate::common::errors::Result<Vec<Uuid>> {
		use crate::infra::db::entities::{wing, wing_group, wing_item};
		use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

		let wing_id = match dependency_type {
			// Pattern 1: Direct - WingLayout ID = Wing ID
			"wing" => dependency_id,

			// Pattern 3: Hierarchical rollup - navigate to parent Wing
			"wing_group" => {
				if let Some(group) = wing_group::Entity::find()
					.filter(wing_group::Column::Uuid.eq(dependency_id))
					.one(db)
					.await?
				{
					wing::Entity::find_by_id(group.wing_id)
						.one(db)
						.await?
						.map(|s| s.uuid)
						.unwrap_or(dependency_id)
				} else {
					dependency_id
				}
			}

			"wing_item" => {
				if let Some(item) = wing_item::Entity::find()
					.filter(wing_item::Column::Uuid.eq(dependency_id))
					.one(db)
					.await?
				{
					wing::Entity::find_by_id(item.wing_id)
						.one(db)
						.await?
						.map(|s| s.uuid)
						.unwrap_or(dependency_id)
				} else {
					dependency_id
				}
			}

			// When location/volume/tag/device changes, invalidate all wings
			// (they will be re-queried with fresh JOINed data)
			"location" | "volume" | "tag" | "device" => {
				// Return all wing IDs to invalidate all layouts
				let all_wings = wing::Entity::find().all(db).await?;
				return Ok(all_wings.into_iter().map(|s| s.uuid).collect());
			}

			_ => return Ok(vec![]),
		};

		Ok(vec![wing_id])
	}

	async fn from_ids(
		db: &sea_orm::DatabaseConnection,
		ids: &[Uuid],
	) -> crate::common::errors::Result<Vec<Self>> {
		WingLayout::from_wing_ids(db, ids).await
	}
}

/// A group with its items
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingGroupWithItems {
	/// The group
	pub group: WingGroup,

	/// Items in this group (sorted by order)
	pub items: Vec<WingItem>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_wing_creation() {
		let wing = Wing::new(
			"Test Wing".to_string(),
			"Folder".to_string(),
			"#FF0000".to_string(),
		);

		assert_eq!(wing.name, "Test Wing");
		assert_eq!(wing.icon, "Folder");
		assert_eq!(wing.color, "#FF0000");
		assert_eq!(wing.order, 0);
	}

	#[test]
	fn test_default_wing() {
		let wing = Wing::create_default();

		assert_eq!(wing.name, "All Devices");
		assert_eq!(wing.icon, "Planet");
		assert_eq!(wing.color, "#3B82F6");
	}

	#[test]
	fn test_color_validation() {
		assert!(Wing::validate_color("#3B82F6"));
		assert!(Wing::validate_color("#FFFFFF"));
		assert!(!Wing::validate_color("3B82F6")); // Missing #
		assert!(!Wing::validate_color("#FFF")); // Too short
		assert!(!Wing::validate_color("#3B82F6A")); // Too long
	}

	#[test]
	fn test_group_creation() {
		let wing_id = Uuid::new_v4();
		let group = WingGroup::create_quick_access(wing_id);

		assert_eq!(group.wing_id, wing_id);
		assert_eq!(group.name, "Quick Access");
		assert_eq!(group.group_type, GroupType::QuickAccess);
		assert!(!group.is_collapsed);
	}

	#[test]
	fn test_device_group_creation() {
		let wing_id = Uuid::new_v4();
		let device_id = Uuid::new_v4();
		let group = WingGroup::create_device(wing_id, device_id, "MacBook Pro".to_string());

		assert_eq!(group.name, "MacBook Pro");
		assert_eq!(group.group_type, GroupType::Device { device_id });
	}

	#[test]
	fn test_item_creation() {
		let wing_id = Uuid::new_v4();
		let group_id = Uuid::new_v4();

		let overview = WingItem::create_overview(wing_id, group_id);
		assert_eq!(overview.item_type, ItemType::Overview);
		assert_eq!(overview.group_id, Some(group_id));

		let recents = WingItem::create_recents(wing_id, group_id);
		assert_eq!(recents.item_type, ItemType::Recents);

		let favorites = WingItem::create_favorites(wing_id, group_id);
		assert_eq!(favorites.item_type, ItemType::Favorites);
	}

	#[test]
	fn test_wing_level_item() {
		let wing_id = Uuid::new_v4();
		let wing_path = crate::domain::WingPath::Physical {
			device_slug: "macbook".to_string(),
			path: "/Users/me/Documents".into(),
		};

		let item = WingItem::create_wing_level_path(wing_id, wing_path);
		assert_eq!(item.wing_id, wing_id);
		assert_eq!(item.group_id, None);
		assert!(matches!(item.item_type, ItemType::Path { .. }));
	}
}

// Register simple resources (single table, no dependencies)
crate::register_resource!(Wing);
crate::register_resource!(WingGroup);
crate::register_resource!(WingItem);

// Register WingLayout as a virtual resource (has dependencies on wing, wing_group, wing_item)
crate::register_resource!(WingLayout, virtual);
