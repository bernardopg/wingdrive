//! Wings: creating a wing with groups and pinned items, reading its layout,
//! reordering, renaming and deleting.

mod helpers;

use helpers::*;
use uuid::Uuid;
use wing_core::{
	domain::{addressing::WingPath, GroupType, ItemType, WingLayout},
	infra::{
		action::{error::ActionError, LibraryAction},
		api::SessionContext,
		query::LibraryQuery,
		sync::{ChangeType, NetworkTransport},
	},
	location::IndexMode,
	ops::wings::{
		AddGroupAction, AddGroupInput, AddItemAction, AddItemInput, DeleteGroupAction,
		DeleteGroupInput, DeleteItemAction, DeleteItemInput, ReorderGroupsAction,
		ReorderGroupsInput, ReorderItemsAction, ReorderItemsInput, UpdateGroupAction,
		UpdateGroupInput, WingCreateAction, WingCreateInput, WingDeleteAction, WingDeleteInput,
		WingGetQuery, WingGetQueryInput, WingLayoutQuery, WingLayoutQueryInput, WingUpdateAction,
		WingUpdateInput, WingsListQuery, WingsListQueryInput,
	},
};

struct Wings {
	harness: IndexingHarness,
}

impl Wings {
	async fn act<A: LibraryAction>(&self, input: A::Input) -> Result<A::Output, ActionError> {
		let action = A::from_input(input).map_err(ActionError::Internal)?;
		self.harness
			.core
			.context
			.get_action_manager()
			.await
			.unwrap()
			.dispatch_library(Some(self.harness.library.id()), action)
			.await
	}

	async fn query<Q: LibraryQuery>(&self, input: Q::Input) -> anyhow::Result<Q::Output> {
		let mut session = SessionContext::device_session(
			wing_core::device::get_current_device_id(),
			wing_core::device::get_current_device_slug(),
		);
		session.current_library_id = Some(self.harness.library.id());
		Ok(Q::from_input(input)?
			.execute(self.harness.core.context.clone(), session)
			.await?)
	}

	async fn layout(&self, wing_id: Uuid) -> anyhow::Result<WingLayout> {
		self.query::<WingLayoutQuery>(WingLayoutQueryInput { wing_id })
			.await
	}

	async fn add_item(
		&self,
		wing_id: Uuid,
		group_id: Option<Uuid>,
		item_type: ItemType,
	) -> Result<Uuid, ActionError> {
		Ok(self
			.act::<AddItemAction>(AddItemInput {
				wing_id,
				group_id,
				item_type,
			})
			.await?
			.item
			.id)
	}
}

async fn wings(test_name: &str) -> anyhow::Result<Wings> {
	let harness = IndexingHarnessBuilder::new(test_name)
		.disable_watcher()
		.build()
		.await?;
	Ok(Wings { harness })
}

fn local(path: std::path::PathBuf) -> WingPath {
	WingPath::Physical {
		device_slug: wing_core::device::get_current_device_slug(),
		path,
	}
}

#[tokio::test]
async fn a_wing_layout_lists_groups_items_and_pinned_files() -> anyhow::Result<()> {
	let wings = wings("wing_layout").await?;
	let location = wings.harness.create_test_location("pins").await?;
	location
		.write_file("notes.txt", "a file beside the folder")
		.await?;
	location.write_file("notes/inside.txt", "inside").await?;
	location
		.write_file("v1.2/readme.md", "dotted folder")
		.await?;
	location.index("Pins", IndexMode::Deep).await?;

	let wing = wings
		.act::<WingCreateAction>(WingCreateInput {
			name: "Work".into(),
			icon: "Briefcase".into(),
			color: "#3B82F6".into(),
		})
		.await?
		.wing;
	let second = wings
		.act::<WingCreateAction>(WingCreateInput {
			name: "Home".into(),
			icon: "House".into(),
			color: "#10B981".into(),
		})
		.await?
		.wing;
	assert_eq!(second.order, wing.order + 1);

	let quick = wings
		.act::<AddGroupAction>(AddGroupInput {
			wing_id: wing.id,
			name: "Quick Access".into(),
			group_type: GroupType::QuickAccess,
		})
		.await?
		.group;
	let tags = wings
		.act::<AddGroupAction>(AddGroupInput {
			wing_id: wing.id,
			name: "Tags".into(),
			group_type: GroupType::Tags,
		})
		.await?
		.group;
	assert_eq!((quick.order, tags.order), (0, 1));

	let recents = wings
		.add_item(wing.id, Some(quick.id), ItemType::Recents)
		.await?;
	let favorites = wings
		.add_item(wing.id, Some(quick.id), ItemType::Favorites)
		.await?;
	let notes_folder = wings
		.add_item(
			wing.id,
			None,
			ItemType::Path {
				wing_path: local(location.path().join("notes")),
			},
		)
		.await?;
	let dotted_folder = wings
		.add_item(
			wing.id,
			None,
			ItemType::Path {
				wing_path: local(location.path().join("v1.2")),
			},
		)
		.await?;
	let unindexed = wings
		.add_item(
			wing.id,
			None,
			ItemType::Path {
				wing_path: local(location.path().join("missing")),
			},
		)
		.await?;

	let layout = wings.layout(wing.id).await?;
	assert_eq!(layout.id, wing.id);
	assert_eq!(layout.wing.name, "Work");
	let item_ids: Vec<_> = layout.wing_items.iter().map(|i| i.id).collect();
	assert_eq!(item_ids, [notes_folder, dotted_folder, unindexed]);
	// A pinned folder resolves to the folder, not to a file of the same name.
	let notes = layout.wing_items[0]
		.resolved_file
		.as_ref()
		.expect("resolved");
	assert_eq!(notes.name, "notes");
	assert_eq!(notes.extension, None);
	let dotted = layout.wing_items[1]
		.resolved_file
		.as_ref()
		.expect("resolved");
	assert_eq!(dotted.name, "v1.2");
	assert!(layout.wing_items[2].resolved_file.is_none());

	let group_ids: Vec<_> = layout.groups.iter().map(|g| g.group.id).collect();
	assert_eq!(group_ids, [quick.id, tags.id]);
	let quick_items: Vec<_> = layout.groups[0].items.iter().map(|i| i.id).collect();
	assert_eq!(quick_items, [recents, favorites]);
	assert!(layout.groups[0]
		.items
		.iter()
		.all(|i| i.resolved_file.is_none()));
	assert!(layout.groups[1].items.is_empty());

	let listed = wings.query::<WingsListQuery>(WingsListQueryInput).await?;
	// A new library already has its default wing, listed first.
	let listed: Vec<_> = listed.wings.iter().map(|s| s.id).collect();
	assert_eq!(listed[listed.len() - 2..], [wing.id, second.id]);
	let fetched = wings
		.query::<WingGetQuery>(WingGetQueryInput { wing_id: wing.id })
		.await?;
	assert_eq!(fetched.wing.icon, "Briefcase");

	wings.harness.shutdown().await
}

#[tokio::test]
async fn groups_and_items_are_reordered_renamed_and_deleted() -> anyhow::Result<()> {
	let wings = wings("wing_editing").await?;
	// A sync service, not started, so the changes meant for other devices are
	// recorded in the peer log and can be read back.
	let (transport, _peer) = MockTransport::new_pair(wings.harness.device_id, Uuid::new_v4());
	wings
		.harness
		.library
		.init_sync_service(
			wings.harness.device_id,
			transport as std::sync::Arc<dyn NetworkTransport>,
		)
		.await?;
	let create = |name: &str| WingCreateInput {
		name: name.into(),
		icon: "Star".into(),
		color: "#F59E0B".into(),
	};
	let wing = wings.act::<WingCreateAction>(create("Main")).await?.wing;
	let other = wings.act::<WingCreateAction>(create("Other")).await?.wing;

	let mut groups = Vec::new();
	for name in ["A", "B", "C"] {
		groups.push(
			wings
				.act::<AddGroupAction>(AddGroupInput {
					wing_id: wing.id,
					name: name.into(),
					group_type: GroupType::Locations,
				})
				.await?
				.group
				.id,
		);
	}
	let foreign_group = wings
		.act::<AddGroupAction>(AddGroupInput {
			wing_id: other.id,
			name: "Elsewhere".into(),
			group_type: GroupType::Devices,
		})
		.await?
		.group
		.id;

	let first = wings
		.add_item(wing.id, Some(groups[0]), ItemType::Overview)
		.await?;
	let second = wings
		.add_item(wing.id, Some(groups[0]), ItemType::FileKinds)
		.await?;
	let pinned = wings.add_item(wing.id, None, ItemType::Sources).await?;

	// An item cannot land in a group of another wing.
	let error = wings
		.add_item(wing.id, Some(foreign_group), ItemType::Recents)
		.await
		.unwrap_err();
	assert!(error.to_string().contains("not found"), "{error}");
	let error = wings
		.add_item(Uuid::new_v4(), None, ItemType::Recents)
		.await
		.unwrap_err();
	assert!(error.to_string().contains("not found"), "{error}");

	wings
		.act::<ReorderGroupsAction>(ReorderGroupsInput {
			wing_id: wing.id,
			group_ids: vec![groups[2], groups[0], groups[1]],
		})
		.await?;
	wings
		.act::<ReorderItemsAction>(ReorderItemsInput {
			group_id: Some(groups[0]),
			item_ids: vec![second, first],
		})
		.await?;
	let layout = wings.layout(wing.id).await?;
	let order: Vec<_> = layout.groups.iter().map(|g| g.group.id).collect();
	assert_eq!(order, [groups[2], groups[0], groups[1]]);
	let items: Vec<_> = layout.groups[1].items.iter().map(|i| i.id).collect();
	assert_eq!(items, [second, first]);

	// Reordering refuses groups of another wing and items of another group.
	let error = wings
		.act::<ReorderGroupsAction>(ReorderGroupsInput {
			wing_id: wing.id,
			group_ids: vec![foreign_group],
		})
		.await
		.unwrap_err();
	assert!(error.to_string().contains("not found"), "{error}");
	let error = wings
		.act::<ReorderItemsAction>(ReorderItemsInput {
			group_id: Some(groups[1]),
			item_ids: vec![first],
		})
		.await
		.unwrap_err();
	assert!(error.to_string().contains("not found"), "{error}");
	// Wing-level items of two wings cannot be renumbered together.
	let foreign_pinned = wings.add_item(other.id, None, ItemType::Recents).await?;
	let error = wings
		.act::<ReorderItemsAction>(ReorderItemsInput {
			group_id: None,
			item_ids: vec![pinned, foreign_pinned],
		})
		.await
		.unwrap_err();
	assert!(error.to_string().contains("not found"), "{error}");
	wings
		.act::<ReorderItemsAction>(ReorderItemsInput {
			group_id: None,
			item_ids: vec![pinned],
		})
		.await?;

	let renamed = wings
		.act::<WingUpdateAction>(WingUpdateInput {
			wing_id: wing.id,
			name: Some("Renamed".into()),
			icon: Some("Folder".into()),
			color: None,
		})
		.await?
		.wing;
	assert_eq!(
		(
			renamed.name.as_str(),
			renamed.icon.as_str(),
			renamed.color.as_str()
		),
		("Renamed", "Folder", "#F59E0B")
	);
	let collapsed = wings
		.act::<UpdateGroupAction>(UpdateGroupInput {
			group_id: groups[1],
			name: Some("Bee".into()),
			is_collapsed: Some(true),
		})
		.await?
		.group;
	assert!(collapsed.is_collapsed);
	assert_eq!(collapsed.name, "Bee");
	assert_eq!(collapsed.wing_id, wing.id);

	wings
		.act::<DeleteItemAction>(DeleteItemInput { item_id: first })
		.await?;
	wings
		.act::<DeleteGroupAction>(DeleteGroupInput {
			group_id: groups[2],
		})
		.await?;
	let layout = wings.layout(wing.id).await?;
	assert_eq!(layout.groups.len(), 2);
	assert_eq!(
		layout.groups[0]
			.items
			.iter()
			.map(|i| i.id)
			.collect::<Vec<_>>(),
		[second]
	);

	let before = wings
		.query::<WingsListQuery>(WingsListQueryInput)
		.await?
		.wings
		.len();
	wings
		.act::<WingDeleteAction>(WingDeleteInput { wing_id: wing.id })
		.await?;
	assert!(wings.layout(wing.id).await.is_err());
	let listed = wings.query::<WingsListQuery>(WingsListQueryInput).await?;
	assert_eq!(listed.wings.len(), before - 1);
	assert!(listed.wings.iter().all(|s| s.id != wing.id));

	for missing in [
		wings
			.act::<WingDeleteAction>(WingDeleteInput { wing_id: wing.id })
			.await
			.map(|_| ()),
		wings
			.act::<DeleteGroupAction>(DeleteGroupInput {
				group_id: groups[2],
			})
			.await
			.map(|_| ()),
		wings
			.act::<DeleteItemAction>(DeleteItemInput { item_id: first })
			.await
			.map(|_| ()),
		wings
			.act::<UpdateGroupAction>(UpdateGroupInput {
				group_id: Uuid::new_v4(),
				name: None,
				is_collapsed: None,
			})
			.await
			.map(|_| ()),
		wings
			.act::<WingUpdateAction>(WingUpdateInput {
				wing_id: Uuid::new_v4(),
				name: None,
				icon: None,
				color: None,
			})
			.await
			.map(|_| ()),
	] {
		let error = missing.unwrap_err();
		assert!(error.to_string().contains("not found"), "{error}");
	}

	// What other devices receive: groups and items name their parents by UUID,
	// since peers number rows differently, and every deletion is recorded.
	let changes = wings
		.harness
		.library
		.sync_service()
		.expect("sync service initialized above")
		.peer_sync()
		.peer_log()
		.get_since(None, None)
		.await?;
	// Wings sync under their pre-rename model and field names, which devices on
	// 3.3.1 and older still expect.
	let mut checked = 0;
	for change in &changes {
		let parents: &[&str] = match change.model_type.as_str() {
			"space_group" => &["space_uuid"],
			"space_item" => &["space_uuid", "group_uuid"],
			_ => continue,
		};
		if change.change_type == ChangeType::Delete {
			continue;
		}
		for parent in parents {
			assert!(
				change.data.get(*parent).is_some(),
				"{} {:?} without {parent}: {}",
				change.model_type,
				change.change_type,
				change.data
			);
		}
		checked += 1;
	}
	assert!(checked > 0, "no wing group or item changes were recorded");
	let deleted: Vec<_> = changes
		.iter()
		.filter(|change| change.change_type == ChangeType::Delete)
		.map(|change| change.record_uuid)
		.collect();
	for id in [first, groups[2], wing.id] {
		assert!(deleted.contains(&id), "deletion of {id} was not recorded");
	}

	wings.harness.shutdown().await
}
