//! Spaces: creating a space with groups and pinned items, reading its layout,
//! reordering, renaming and deleting.

mod helpers;

use helpers::*;
use uuid::Uuid;
use wing_core::{
	domain::{addressing::WingPath, GroupType, ItemType, SpaceLayout},
	infra::{
		action::{error::ActionError, LibraryAction},
		api::SessionContext,
		query::LibraryQuery,
		sync::{ChangeType, NetworkTransport},
	},
	location::IndexMode,
	ops::spaces::{
		AddGroupAction, AddGroupInput, AddItemAction, AddItemInput, DeleteGroupAction,
		DeleteGroupInput, DeleteItemAction, DeleteItemInput, ReorderGroupsAction,
		ReorderGroupsInput, ReorderItemsAction, ReorderItemsInput, SpaceCreateAction,
		SpaceCreateInput, SpaceDeleteAction, SpaceDeleteInput, SpaceGetQuery, SpaceGetQueryInput,
		SpaceLayoutQuery, SpaceLayoutQueryInput, SpaceUpdateAction, SpaceUpdateInput,
		SpacesListQuery, SpacesListQueryInput, UpdateGroupAction, UpdateGroupInput,
	},
};

struct Spaces {
	harness: IndexingHarness,
}

impl Spaces {
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

	async fn layout(&self, space_id: Uuid) -> anyhow::Result<SpaceLayout> {
		self.query::<SpaceLayoutQuery>(SpaceLayoutQueryInput { space_id })
			.await
	}

	async fn add_item(
		&self,
		space_id: Uuid,
		group_id: Option<Uuid>,
		item_type: ItemType,
	) -> Result<Uuid, ActionError> {
		Ok(self
			.act::<AddItemAction>(AddItemInput {
				space_id,
				group_id,
				item_type,
			})
			.await?
			.item
			.id)
	}
}

async fn spaces(test_name: &str) -> anyhow::Result<Spaces> {
	let harness = IndexingHarnessBuilder::new(test_name)
		.disable_watcher()
		.build()
		.await?;
	Ok(Spaces { harness })
}

fn local(path: std::path::PathBuf) -> WingPath {
	WingPath::Physical {
		device_slug: wing_core::device::get_current_device_slug(),
		path: path.into(),
	}
}

#[tokio::test]
async fn a_space_layout_lists_groups_items_and_pinned_files() -> anyhow::Result<()> {
	let spaces = spaces("space_layout").await?;
	let location = spaces.harness.create_test_location("pins").await?;
	location
		.write_file("notes.txt", "a file beside the folder")
		.await?;
	location.write_file("notes/inside.txt", "inside").await?;
	location
		.write_file("v1.2/readme.md", "dotted folder")
		.await?;
	location.index("Pins", IndexMode::Deep).await?;

	let space = spaces
		.act::<SpaceCreateAction>(SpaceCreateInput {
			name: "Work".into(),
			icon: "Briefcase".into(),
			color: "#3B82F6".into(),
		})
		.await?
		.space;
	let second = spaces
		.act::<SpaceCreateAction>(SpaceCreateInput {
			name: "Home".into(),
			icon: "House".into(),
			color: "#10B981".into(),
		})
		.await?
		.space;
	assert_eq!(second.order, space.order + 1);

	let quick = spaces
		.act::<AddGroupAction>(AddGroupInput {
			space_id: space.id,
			name: "Quick Access".into(),
			group_type: GroupType::QuickAccess,
		})
		.await?
		.group;
	let tags = spaces
		.act::<AddGroupAction>(AddGroupInput {
			space_id: space.id,
			name: "Tags".into(),
			group_type: GroupType::Tags,
		})
		.await?
		.group;
	assert_eq!((quick.order, tags.order), (0, 1));

	let recents = spaces
		.add_item(space.id, Some(quick.id), ItemType::Recents)
		.await?;
	let favorites = spaces
		.add_item(space.id, Some(quick.id), ItemType::Favorites)
		.await?;
	let notes_folder = spaces
		.add_item(
			space.id,
			None,
			ItemType::Path {
				wing_path: local(location.path().join("notes")),
			},
		)
		.await?;
	let dotted_folder = spaces
		.add_item(
			space.id,
			None,
			ItemType::Path {
				wing_path: local(location.path().join("v1.2")),
			},
		)
		.await?;
	let unindexed = spaces
		.add_item(
			space.id,
			None,
			ItemType::Path {
				wing_path: local(location.path().join("missing")),
			},
		)
		.await?;

	let layout = spaces.layout(space.id).await?;
	assert_eq!(layout.id, space.id);
	assert_eq!(layout.space.name, "Work");
	let item_ids: Vec<_> = layout.space_items.iter().map(|i| i.id).collect();
	assert_eq!(item_ids, [notes_folder, dotted_folder, unindexed]);
	// A pinned folder resolves to the folder, not to a file of the same name.
	let notes = layout.space_items[0]
		.resolved_file
		.as_ref()
		.expect("resolved");
	assert_eq!(notes.name, "notes");
	assert_eq!(notes.extension, None);
	let dotted = layout.space_items[1]
		.resolved_file
		.as_ref()
		.expect("resolved");
	assert_eq!(dotted.name, "v1.2");
	assert!(layout.space_items[2].resolved_file.is_none());

	let group_ids: Vec<_> = layout.groups.iter().map(|g| g.group.id).collect();
	assert_eq!(group_ids, [quick.id, tags.id]);
	let quick_items: Vec<_> = layout.groups[0].items.iter().map(|i| i.id).collect();
	assert_eq!(quick_items, [recents, favorites]);
	assert!(layout.groups[0]
		.items
		.iter()
		.all(|i| i.resolved_file.is_none()));
	assert!(layout.groups[1].items.is_empty());

	let listed = spaces
		.query::<SpacesListQuery>(SpacesListQueryInput)
		.await?;
	// A new library already has its default space, listed first.
	let listed: Vec<_> = listed.spaces.iter().map(|s| s.id).collect();
	assert_eq!(listed[listed.len() - 2..], [space.id, second.id]);
	let fetched = spaces
		.query::<SpaceGetQuery>(SpaceGetQueryInput { space_id: space.id })
		.await?;
	assert_eq!(fetched.space.icon, "Briefcase");

	spaces.harness.shutdown().await
}

#[tokio::test]
async fn groups_and_items_are_reordered_renamed_and_deleted() -> anyhow::Result<()> {
	let spaces = spaces("space_editing").await?;
	// A sync service, not started, so the changes meant for other devices are
	// recorded in the peer log and can be read back.
	let (transport, _peer) = MockTransport::new_pair(spaces.harness.device_id, Uuid::new_v4());
	spaces
		.harness
		.library
		.init_sync_service(
			spaces.harness.device_id,
			transport as std::sync::Arc<dyn NetworkTransport>,
		)
		.await?;
	let create = |name: &str| SpaceCreateInput {
		name: name.into(),
		icon: "Star".into(),
		color: "#F59E0B".into(),
	};
	let space = spaces.act::<SpaceCreateAction>(create("Main")).await?.space;
	let other = spaces
		.act::<SpaceCreateAction>(create("Other"))
		.await?
		.space;

	let mut groups = Vec::new();
	for name in ["A", "B", "C"] {
		groups.push(
			spaces
				.act::<AddGroupAction>(AddGroupInput {
					space_id: space.id,
					name: name.into(),
					group_type: GroupType::Locations,
				})
				.await?
				.group
				.id,
		);
	}
	let foreign_group = spaces
		.act::<AddGroupAction>(AddGroupInput {
			space_id: other.id,
			name: "Elsewhere".into(),
			group_type: GroupType::Devices,
		})
		.await?
		.group
		.id;

	let first = spaces
		.add_item(space.id, Some(groups[0]), ItemType::Overview)
		.await?;
	let second = spaces
		.add_item(space.id, Some(groups[0]), ItemType::FileKinds)
		.await?;
	let pinned = spaces.add_item(space.id, None, ItemType::Sources).await?;

	// An item cannot land in a group of another space.
	let error = spaces
		.add_item(space.id, Some(foreign_group), ItemType::Recents)
		.await
		.unwrap_err();
	assert!(error.to_string().contains("not found"), "{error}");
	let error = spaces
		.add_item(Uuid::new_v4(), None, ItemType::Recents)
		.await
		.unwrap_err();
	assert!(error.to_string().contains("not found"), "{error}");

	spaces
		.act::<ReorderGroupsAction>(ReorderGroupsInput {
			space_id: space.id,
			group_ids: vec![groups[2], groups[0], groups[1]],
		})
		.await?;
	spaces
		.act::<ReorderItemsAction>(ReorderItemsInput {
			group_id: Some(groups[0]),
			item_ids: vec![second, first],
		})
		.await?;
	let layout = spaces.layout(space.id).await?;
	let order: Vec<_> = layout.groups.iter().map(|g| g.group.id).collect();
	assert_eq!(order, [groups[2], groups[0], groups[1]]);
	let items: Vec<_> = layout.groups[1].items.iter().map(|i| i.id).collect();
	assert_eq!(items, [second, first]);

	// Reordering refuses groups of another space and items of another group.
	let error = spaces
		.act::<ReorderGroupsAction>(ReorderGroupsInput {
			space_id: space.id,
			group_ids: vec![foreign_group],
		})
		.await
		.unwrap_err();
	assert!(error.to_string().contains("not found"), "{error}");
	let error = spaces
		.act::<ReorderItemsAction>(ReorderItemsInput {
			group_id: Some(groups[1]),
			item_ids: vec![first],
		})
		.await
		.unwrap_err();
	assert!(error.to_string().contains("not found"), "{error}");
	// Space-level items of two spaces cannot be renumbered together.
	let foreign_pinned = spaces.add_item(other.id, None, ItemType::Recents).await?;
	let error = spaces
		.act::<ReorderItemsAction>(ReorderItemsInput {
			group_id: None,
			item_ids: vec![pinned, foreign_pinned],
		})
		.await
		.unwrap_err();
	assert!(error.to_string().contains("not found"), "{error}");
	spaces
		.act::<ReorderItemsAction>(ReorderItemsInput {
			group_id: None,
			item_ids: vec![pinned],
		})
		.await?;

	let renamed = spaces
		.act::<SpaceUpdateAction>(SpaceUpdateInput {
			space_id: space.id,
			name: Some("Renamed".into()),
			icon: Some("Folder".into()),
			color: None,
		})
		.await?
		.space;
	assert_eq!(
		(
			renamed.name.as_str(),
			renamed.icon.as_str(),
			renamed.color.as_str()
		),
		("Renamed", "Folder", "#F59E0B")
	);
	let collapsed = spaces
		.act::<UpdateGroupAction>(UpdateGroupInput {
			group_id: groups[1],
			name: Some("Bee".into()),
			is_collapsed: Some(true),
		})
		.await?
		.group;
	assert!(collapsed.is_collapsed);
	assert_eq!(collapsed.name, "Bee");
	assert_eq!(collapsed.space_id, space.id);

	spaces
		.act::<DeleteItemAction>(DeleteItemInput { item_id: first })
		.await?;
	spaces
		.act::<DeleteGroupAction>(DeleteGroupInput {
			group_id: groups[2],
		})
		.await?;
	let layout = spaces.layout(space.id).await?;
	assert_eq!(layout.groups.len(), 2);
	assert_eq!(
		layout.groups[0]
			.items
			.iter()
			.map(|i| i.id)
			.collect::<Vec<_>>(),
		[second]
	);

	let before = spaces
		.query::<SpacesListQuery>(SpacesListQueryInput)
		.await?
		.spaces
		.len();
	spaces
		.act::<SpaceDeleteAction>(SpaceDeleteInput { space_id: space.id })
		.await?;
	assert!(spaces.layout(space.id).await.is_err());
	let listed = spaces
		.query::<SpacesListQuery>(SpacesListQueryInput)
		.await?;
	assert_eq!(listed.spaces.len(), before - 1);
	assert!(listed.spaces.iter().all(|s| s.id != space.id));

	for missing in [
		spaces
			.act::<SpaceDeleteAction>(SpaceDeleteInput { space_id: space.id })
			.await
			.map(|_| ()),
		spaces
			.act::<DeleteGroupAction>(DeleteGroupInput {
				group_id: groups[2],
			})
			.await
			.map(|_| ()),
		spaces
			.act::<DeleteItemAction>(DeleteItemInput { item_id: first })
			.await
			.map(|_| ()),
		spaces
			.act::<UpdateGroupAction>(UpdateGroupInput {
				group_id: Uuid::new_v4(),
				name: None,
				is_collapsed: None,
			})
			.await
			.map(|_| ()),
		spaces
			.act::<SpaceUpdateAction>(SpaceUpdateInput {
				space_id: Uuid::new_v4(),
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
	let changes = spaces
		.harness
		.library
		.sync_service()
		.expect("sync service initialized above")
		.peer_sync()
		.peer_log()
		.get_since(None, None)
		.await?;
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
	assert!(checked > 0, "no space group or item changes were recorded");
	let deleted: Vec<_> = changes
		.iter()
		.filter(|change| change.change_type == ChangeType::Delete)
		.map(|change| change.record_uuid)
		.collect();
	for id in [first, groups[2], space.id] {
		assert!(deleted.contains(&id), "deletion of {id} was not recorded");
	}

	spaces.harness.shutdown().await
}
