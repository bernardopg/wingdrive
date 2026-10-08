//! Persistent indexing and ephemeral browsing must agree on entry identities.

mod helpers;

use anyhow::Result;
use helpers::{register_device, wait_for_indexing, IndexingHarnessBuilder};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::path::PathBuf;
use tokio::time::{sleep, timeout, Duration};
use uuid::Uuid;
use wing_core::{
	domain::addressing::WingPath,
	infra::{
		action::LibraryAction,
		api::SessionContext,
		db::entities::{device, entry, location},
		event::Event,
		query::LibraryQuery,
	},
	location::{IndexMode, LocationManager},
	ops::{
		files::query::directory_listing::DirectoryListingQuery,
		indexing::{
			database_storage::EntryMetadata,
			ephemeral::{
				extract_persistent_uuids_for_path, get_or_resolve_uuid, DatabaseUuidLookup,
				EphemeralIndex,
			},
			state::EntryKind,
			IndexScope, IndexerJob, IndexerJobConfig,
		},
		tags::{
			apply::{action::ApplyTagsAction, input::ApplyTagsInput},
			create::{action::CreateTagAction, input::CreateTagInput},
		},
	},
};

fn metadata(path: PathBuf) -> EntryMetadata {
	EntryMetadata {
		path,
		kind: EntryKind::File,
		size: 5,
		modified: None,
		accessed: None,
		created: None,
		inode: None,
		permissions: None,
		is_hidden: false,
	}
}

#[tokio::test]
async fn persistent_index_and_ephemeral_browse_share_uuids() -> Result<()> {
	let harness = IndexingHarnessBuilder::new("uuid_reconciliation")
		.disable_watcher()
		.build()
		.await?;
	let location = harness.create_test_location("managed").await?;
	let direct = location.write_file("direct.txt", "hello").await?;
	let nested = location.write_file("sub/nested.txt", "world").await?;
	let _handle = location.index("Managed", IndexMode::Deep).await?;

	let root = location.path();
	let stored = extract_persistent_uuids_for_path(harness.library.db().conn(), root).await?;
	let direct_uuid = *stored.get(&direct).expect("indexed direct file");
	let nested_uuid = *stored.get(&nested).expect("indexed nested file");
	assert_ne!(direct_uuid, nested_uuid);

	let mut browse = EphemeralIndex::default();
	for path in [&direct, &nested] {
		browse.add_entry(path.clone(), Uuid::new_v4(), metadata(path.clone()))?;
	}
	let library_id = harness.library.id();
	let before = browse.get_or_assign_uuid_scoped(library_id, &direct);
	assert_ne!(before, direct_uuid);
	let result = browse.reconcile_with_persistent(library_id, root, &stored);
	assert_eq!(
		browse.get_entry_uuid_scoped(library_id, &direct),
		Some(direct_uuid)
	);
	assert_eq!(
		browse.get_entry_uuid_scoped(library_id, &nested),
		Some(nested_uuid)
	);
	assert!(result
		.changes
		.iter()
		.any(|(path, old, new)| path == &direct && *old == Some(before) && *new == direct_uuid));

	let other_library = Uuid::new_v4();
	assert_ne!(
		browse.get_or_assign_uuid_scoped(other_library, &direct),
		direct_uuid
	);
	assert_eq!(
		browse.get_entry_uuid_scoped(library_id, &direct),
		Some(direct_uuid)
	);

	let lookup = DatabaseUuidLookup::new(harness.library.db().conn().clone());
	assert_eq!(
		get_or_resolve_uuid(&mut browse, library_id, &nested, &lookup).await,
		nested_uuid
	);
	harness.shutdown().await?;
	Ok(())
}

#[tokio::test]
async fn cached_listing_resolves_uuid_before_background_reconciliation() -> Result<()> {
	let harness = IndexingHarnessBuilder::new("cached_uuid_listing")
		.disable_watcher()
		.build()
		.await?;
	let volume = harness.temp_path().join("volume");
	tokio::fs::create_dir_all(&volume).await?;
	let root = volume.join("managed");
	tokio::fs::create_dir_all(&root).await?;
	tokio::fs::write(root.join("file.txt"), "content").await?;
	harness
		.add_and_index_location(&root, "Managed", IndexMode::Deep)
		.await?;
	let persisted = extract_persistent_uuids_for_path(harness.library.db().conn(), &root).await?;
	let root_uuid = *persisted.get(&root).expect("persistent root UUID");
	let root_entry = entry::Entity::find()
		.filter(entry::Column::Uuid.eq(root_uuid))
		.one(harness.library.db().conn())
		.await?
		.expect("persistent root entry");
	let actions = harness
		.core
		.context
		.get_action_manager()
		.await
		.expect("action manager");
	let tag = actions
		.dispatch_library(
			Some(harness.library.id()),
			CreateTagAction::from_input(CreateTagInput::simple("Cached root".into()))
				.map_err(anyhow::Error::msg)?,
		)
		.await?
		.tag_id;
	actions
		.dispatch_library(
			Some(harness.library.id()),
			ApplyTagsAction::from_input(ApplyTagsInput::user_tags_entry(
				vec![root_entry.id],
				vec![tag],
			))
			.map_err(anyhow::Error::msg)?,
		)
		.await?;

	let cache = harness.core.context.ephemeral_cache();
	let index = cache.create_for_indexing(volume.clone(), IndexScope::Recursive);
	let temporary = Uuid::new_v4();
	let mut root_metadata = metadata(root.clone());
	root_metadata.kind = EntryKind::Directory;
	index
		.write()
		.await
		.add_entry(root.clone(), temporary, root_metadata)?;
	cache.mark_indexing_complete(&volume, IndexScope::Recursive);
	let mut events = harness.library.event_bus().subscribe();
	let session = SessionContext::device_session(
		harness.device_id,
		wing_core::device::get_current_device_slug(),
	)
	.with_library(harness.library.id());
	let listing = DirectoryListingQuery::new(WingPath::local(volume.clone()))
		.execute(harness.core.context.clone(), session)
		.await?;
	assert_eq!(listing.files.len(), 1);
	assert_eq!(listing.files[0].id, root_uuid);
	assert!(listing.files[0].tags.iter().any(|t| t.id == tag));
	assert_eq!(
		index
			.read()
			.await
			.get_entry_uuid_scoped(harness.library.id(), &root),
		Some(root_uuid)
	);
	let event = timeout(Duration::from_secs(2), async {
		loop {
			if let Event::ResourceChanged {
				resource, metadata, ..
			} = events.recv().await?
			{
				if resource["id"].as_str() == Some(&root_uuid.to_string()[..]) {
					break Ok::<_, anyhow::Error>(metadata.expect("reconciliation metadata"));
				}
			}
		}
	})
	.await??;
	assert_eq!(event.alternate_ids, vec![temporary]);
	harness.shutdown().await?;
	Ok(())
}

#[tokio::test]
async fn volume_scan_reconciles_two_locations_without_cross_library_leakage() -> Result<()> {
	let harness = IndexingHarnessBuilder::new("volume_uuid_reconciliation")
		.disable_watcher()
		.build()
		.await?;
	let volume = harness.temp_path().join("volume");
	let first = volume.join("first");
	let second = volume.join("second");
	tokio::fs::create_dir_all(&first).await?;
	tokio::fs::create_dir_all(&second).await?;
	let first_file = first.join("first.txt");
	let second_file = second.join("second.txt");
	tokio::fs::write(&first_file, "first").await?;
	tokio::fs::write(&second_file, "second").await?;
	harness
		.add_and_index_location(&first, "First", IndexMode::Deep)
		.await?;
	harness
		.add_and_index_location(&second, "Second", IndexMode::Deep)
		.await?;
	let first_db = extract_persistent_uuids_for_path(harness.library.db().conn(), &volume).await?;
	let a = *first_db.get(&first_file).expect("first indexed location");
	let b = *first_db.get(&second_file).expect("second indexed location");

	let other = harness
		.core
		.libraries
		.create_library("Overlapping library", None, harness.core.context.clone())
		.await?;
	register_device(&other, harness.device_id, "overlap-device").await?;
	let device_id = device::Entity::find()
		.filter(device::Column::Uuid.eq(harness.device_id))
		.one(other.db().conn())
		.await?
		.expect("device in second library")
		.id;
	let manager = LocationManager::new((*harness.core.events).clone());
	let (location_uuid, _) = manager
		.add_location(
			other.clone(),
			WingPath::local(first.clone()),
			Some("Overlapping first".into()),
			device_id,
			IndexMode::Deep,
			None,
			None,
			&harness.core.context.volume_manager,
		)
		.await?;
	let location_id = location::Entity::find()
		.filter(location::Column::Uuid.eq(location_uuid))
		.one(other.db().conn())
		.await?
		.expect("second library location")
		.id;
	wait_for_indexing(&other, location_id, Duration::from_secs(30)).await?;
	let other_db = extract_persistent_uuids_for_path(other.db().conn(), &volume).await?;
	let other_a = *other_db.get(&first_file).expect("overlapping entry");
	assert_ne!(a, other_a);

	let cache = harness.core.context.ephemeral_cache();
	let index = cache.create_for_indexing(volume.clone(), IndexScope::Recursive);
	for library in [&harness.library, &other] {
		let mut job = IndexerJob::new(IndexerJobConfig::ephemeral_browse(
			WingPath::local(volume.clone()),
			IndexScope::Recursive,
			true,
		));
		job.set_ephemeral_index(index.clone());
		library.jobs().dispatch(job).await?.wait().await?;
	}
	timeout(Duration::from_secs(10), async {
		loop {
			let read = index.read().await;
			if read.get_entry_uuid_scoped(harness.library.id(), &first_file) == Some(a)
				&& read.get_entry_uuid_scoped(harness.library.id(), &second_file) == Some(b)
				&& read.get_entry_uuid_scoped(other.id(), &first_file) == Some(other_a)
			{
				assert_ne!(
					read.get_entry_uuid_scoped(other.id(), &second_file),
					Some(b)
				);
				break;
			}
			drop(read);
			sleep(Duration::from_millis(25)).await;
		}
	})
	.await?;
	harness.shutdown().await?;
	Ok(())
}

#[tokio::test]
async fn live_reconciliation_emits_persistent_uuid_and_tags() -> Result<()> {
	let harness = IndexingHarnessBuilder::new("live_uuid_reconciliation")
		.disable_watcher()
		.build()
		.await?;
	let location = harness.create_test_location("managed").await?;
	let file = location.write_file("tagged.txt", "hello").await?;
	let _handle = location.index("Managed", IndexMode::Deep).await?;
	let root = location.path().to_path_buf();
	let stored = extract_persistent_uuids_for_path(harness.library.db().conn(), &root).await?;
	let uuid = *stored.get(&file).expect("indexed file UUID");
	let entry = entry::Entity::find()
		.filter(entry::Column::Uuid.eq(uuid))
		.one(harness.library.db().conn())
		.await?
		.expect("indexed entry");

	let actions = harness
		.core
		.context
		.get_action_manager()
		.await
		.expect("action manager");
	let tag = actions
		.dispatch_library(
			Some(harness.library.id()),
			CreateTagAction::from_input(CreateTagInput::simple("Reconciled".into()))
				.map_err(anyhow::Error::msg)?,
		)
		.await?
		.tag_id;
	actions
		.dispatch_library(
			Some(harness.library.id()),
			ApplyTagsAction::from_input(ApplyTagsInput::user_tags_entry(vec![entry.id], vec![tag]))
				.map_err(anyhow::Error::msg)?,
		)
		.await?;

	let mut events = harness.library.event_bus().subscribe();
	let cache = harness.core.context.ephemeral_cache();
	let index = cache.create_for_indexing(root.clone(), IndexScope::Recursive);
	let mut job = IndexerJob::new(IndexerJobConfig::ephemeral_browse(
		WingPath::local(root.clone()),
		IndexScope::Recursive,
		false,
	));
	job.set_ephemeral_index(index.clone());
	harness.library.jobs().dispatch(job).await?.wait().await?;

	let (old_uuid, event_tags) = timeout(Duration::from_secs(10), async {
		loop {
			if let Event::ResourceChanged {
				resource_type,
				resource,
				metadata,
			} = events.recv().await?
			{
				if resource_type == "file" && resource["id"].as_str() == Some(&uuid.to_string()[..])
				{
					let old = metadata
						.and_then(|m| m.alternate_ids.first().copied())
						.expect("old UUID alias");
					let tags = resource["tags"].as_array().cloned().expect("tags array");
					break Ok::<_, anyhow::Error>((old, tags));
				}
			}
		}
	})
	.await??;
	assert_ne!(old_uuid, uuid);
	assert!(
		event_tags
			.iter()
			.any(|value| value["id"].as_str() == Some(&tag.to_string()[..])),
		"reconciliation event must carry persistent tags: {event_tags:?}"
	);
	assert_eq!(
		index
			.read()
			.await
			.get_entry_uuid_scoped(harness.library.id(), &file),
		Some(uuid)
	);
	harness.shutdown().await?;
	Ok(())
}
