//! Targeted coverage for database/indexing reconciliation (COV-04).
//!
//! These tests exercise temp SQLite-backed indexing flows with isolated fixtures:
//! reindex idempotency, tag preservation, rename/delete/subtree reconciliation,
//! hidden entries, symlinks, and concurrent reindex dispatch.

mod helpers;

use anyhow::Result;
use helpers::IndexingHarnessBuilder;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use std::collections::HashSet;
use uuid::Uuid;
use wing_core::{
	infra::{
		action::LibraryAction,
		db::entities::{entry, entry_closure, tag, user_metadata_tag},
	},
	location::IndexMode,
	ops::tags::{
		apply::{action::ApplyTagsAction, input::ApplyTagsInput},
		create::{action::CreateTagAction, input::CreateTagInput},
	},
};

async fn apply_simple_tag(
	harness: &helpers::IndexingHarness,
	entry_id: i32,
	name: &str,
) -> Result<Uuid> {
	let actions = harness
		.core
		.context
		.get_action_manager()
		.await
		.expect("action manager");
	let tag_id = actions
		.dispatch_library(
			Some(harness.library.id()),
			CreateTagAction::from_input(CreateTagInput::simple(name.into()))
				.map_err(anyhow::Error::msg)?,
		)
		.await?
		.tag_id;
	actions
		.dispatch_library(
			Some(harness.library.id()),
			ApplyTagsAction::from_input(ApplyTagsInput::user_tags_entry(
				vec![entry_id],
				vec![tag_id],
			))
			.map_err(anyhow::Error::msg)?,
		)
		.await?;
	Ok(tag_id)
}

async fn entry_has_user_tag(
	harness: &helpers::IndexingHarness,
	entry_uuid: Uuid,
	tag_uuid: Uuid,
) -> Result<bool> {
	use sea_orm::QuerySelect;
	let db = harness.library.db().conn();
	let metadata_ids: Vec<i32> = wing_core::infra::db::entities::user_metadata::Entity::find()
		.filter(wing_core::infra::db::entities::user_metadata::Column::EntryUuid.eq(entry_uuid))
		.select_only()
		.column(wing_core::infra::db::entities::user_metadata::Column::Id)
		.into_tuple()
		.all(db)
		.await?;
	if metadata_ids.is_empty() {
		return Ok(false);
	}
	let tag_db_id: Option<i32> = tag::Entity::find()
		.filter(tag::Column::Uuid.eq(tag_uuid))
		.select_only()
		.column(tag::Column::Id)
		.into_tuple()
		.one(db)
		.await?;
	let Some(tag_db_id) = tag_db_id else {
		return Ok(false);
	};
	let count = user_metadata_tag::Entity::find()
		.filter(user_metadata_tag::Column::UserMetadataId.is_in(metadata_ids))
		.filter(user_metadata_tag::Column::TagId.eq(tag_db_id))
		.count(db)
		.await?;
	Ok(count > 0)
}

#[tokio::test]
async fn reindex_is_idempotent_and_preserves_user_tags() -> Result<()> {
	let harness = IndexingHarnessBuilder::new("cov04_reindex_idempotent")
		.build()
		.await?;
	let location = harness.create_test_location("loc").await?;
	location.write_file("keep.txt", "hello").await?;
	location.write_file("sub/nested.txt", "world").await?;
	let handle = location.index("Loc", IndexMode::Deep).await?;

	let before = handle.get_all_entries().await?;
	let keep = before
		.iter()
		.find(|e| e.name == "keep")
		.expect("keep.txt indexed");
	let nested = before
		.iter()
		.find(|e| e.name == "nested")
		.expect("nested.txt indexed");
	let tag_id = apply_simple_tag(&harness, keep.id, "cov04-keep").await?;

	handle.reindex().await?;
	let after_first = handle.get_all_entries().await?;
	handle.reindex().await?;
	let after_second = handle.get_all_entries().await?;

	let ids_first: HashSet<i32> = after_first.iter().map(|e| e.id).collect();
	let ids_second: HashSet<i32> = after_second.iter().map(|e| e.id).collect();
	assert_eq!(ids_first, ids_second, "reindex must not duplicate entries");
	assert_eq!(
		after_first.len(),
		after_second.len(),
		"entry count must be stable across reindex"
	);

	let keep_after = after_second
		.iter()
		.find(|e| e.id == keep.id)
		.expect("tagged entry survives reindex");
	assert_eq!(keep_after.uuid, keep.uuid);
	let nested_after = after_second
		.iter()
		.find(|e| e.id == nested.id)
		.expect("nested entry survives reindex");
	assert_eq!(nested_after.uuid, nested.uuid);

	let keep_uuid = keep_after.uuid.expect("indexed entry has uuid");
	assert!(
		entry_has_user_tag(&harness, keep_uuid, tag_id).await?,
		"user tags must survive reindex"
	);

	// Tag row itself should remain present.
	let tag_row = tag::Entity::find()
		.filter(tag::Column::Uuid.eq(tag_id))
		.one(harness.library.db().conn())
		.await?;
	assert!(tag_row.is_some(), "tag row must not be removed by reindex");

	harness.shutdown().await
}

#[tokio::test]
async fn rename_delete_and_subtree_reconcile_database() -> Result<()> {
	let harness = IndexingHarnessBuilder::new("cov04_subtree_reconcile")
		.build()
		.await?;
	let location = harness.create_test_location("loc").await?;
	location.write_file("dir/a.txt", "a").await?;
	location.write_file("dir/b.txt", "b").await?;
	location.write_file("dir/sub/c.txt", "c").await?;
	location.write_file("root.txt", "root").await?;
	let handle = location.index("Loc", IndexMode::Deep).await?;

	handle.verify_closure_table_integrity().await?;

	// Rename directory subtree.
	handle.move_file("dir", "renamed").await?;
	handle.reindex().await?;
	handle.verify_closure_table_integrity().await?;

	let entries = handle.get_all_entries().await?;
	assert!(entries.iter().any(|e| e.name == "renamed"));
	assert!(!entries.iter().any(|e| e.name == "dir"));
	for name in ["a", "b", "c", "sub"] {
		assert!(
			entries.iter().any(|e| e.name == name),
			"subtree entry {name} should survive directory rename"
		);
	}

	// Delete subtree and verify DB rows are gone, not orphaned.
	handle.delete_file("renamed/sub/c.txt").await?;
	std::fs::remove_dir_all(handle.path.join("renamed"))?;
	handle.reindex().await?;
	handle.verify_closure_table_integrity().await?;

	let entries_after = handle.get_all_entries().await?;
	for name in ["renamed", "a", "b", "c", "sub"] {
		assert!(
			!entries_after.iter().any(|e| e.name == name),
			"deleted subtree entry {name} must be removed"
		);
	}
	assert!(entries_after.iter().any(|e| e.name == "root"));

	// Closure rows should not reference missing descendants for this location.
	let loc_id = handle
		.entry_id
		.ok_or_else(|| anyhow::anyhow!("location has no entry_id"))?;
	let closure_rows = entry_closure::Entity::find()
		.filter(entry_closure::Column::AncestorId.eq(loc_id))
		.all(harness.library.db().conn())
		.await?;
	let entry_ids: HashSet<i32> = entries_after.iter().map(|e| e.id).collect();
	for row in closure_rows {
		assert!(
			entry_ids.contains(&row.descendant_id),
			"closure descendant {} missing from entries",
			row.descendant_id
		);
	}

	harness.shutdown().await
}

#[tokio::test]
async fn hidden_files_and_symlinks_are_handled_without_duplicates() -> Result<()> {
	let harness = IndexingHarnessBuilder::new("cov04_hidden_symlink")
		.build()
		.await?;
	let location = harness.create_test_location("loc").await?;
	location.write_file("visible.txt", "ok").await?;
	location.write_file(".hidden_file", "secret").await?;
	location.create_dir(".hidden_dir").await?;
	location
		.write_file(".hidden_dir/inside.txt", "inside")
		.await?;

	// Symlink inside location pointing to another file in same location.
	#[cfg(unix)]
	{
		let target = location.path().join("visible.txt");
		let link = location.path().join("link_to_visible.txt");
		std::os::unix::fs::symlink(&target, &link)?;
	}

	let handle = location.index("Loc", IndexMode::Deep).await?;
	handle.reindex().await?;
	let entries = handle.get_all_entries().await?;

	// No duplicates by path identity (id is unique, but name+parent could expose dup bugs).
	let ids: HashSet<i32> = entries.iter().map(|e| e.id).collect();
	assert_eq!(ids.len(), entries.len(), "entry IDs must be unique");

	// Visible file always present.
	assert!(entries.iter().any(|e| e.name == "visible"));

	// Hidden handling depends on indexer rules/config; assert no crash and stable count.
	let count_after = entries.len();
	handle.reindex().await?;
	let entries_after = handle.get_all_entries().await?;
	assert_eq!(
		count_after,
		entries_after.len(),
		"hidden/symlink indexing must be stable across reindex"
	);

	#[cfg(unix)]
	{
		// If symlink was indexed, it should not create duplicate visible content rows.
		let visible_count = entries_after.iter().filter(|e| e.name == "visible").count();
		assert_eq!(visible_count, 1, "visible.txt should not be duplicated");
	}

	harness.shutdown().await
}

#[tokio::test]
async fn concurrent_reindex_dispatches_do_not_duplicate_entries() -> Result<()> {
	let harness = IndexingHarnessBuilder::new("cov04_concurrent_reindex")
		.build()
		.await?;
	let location = harness.create_test_location("loc").await?;
	for i in 0..20 {
		location
			.write_file(&format!("dir/file_{i}.txt"), &format!("content {i}"))
			.await?;
	}
	let handle = location.index("Loc", IndexMode::Deep).await?;
	let before = handle.get_all_entries().await?;

	// Dispatch two reindex jobs back-to-back and wait for both. The job system may
	// serialize or dedupe; either way, final DB state must have no duplicates.
	use wing_core::{
		domain::addressing::WingPath,
		ops::indexing::{IndexerJob, IndexerJobConfig},
	};
	let location_record = entry::Entity::find_by_id(
		handle
			.entry_id
			.ok_or_else(|| anyhow::anyhow!("location has no entry_id"))?,
	)
	.one(harness.library.db().conn())
	.await?
	.ok_or_else(|| anyhow::anyhow!("location root entry missing"))?;
	let _ = location_record;

	// Rebuild config from location handle path/mode.
	let mode = wing_core::domain::location::IndexMode::Deep;
	let cfg = IndexerJobConfig::new(handle.uuid, WingPath::local(&handle.path), mode);
	let job1 = IndexerJob::new(cfg.clone());
	let job2 = IndexerJob::new(cfg);
	let h1 = harness.library.jobs().dispatch(job1).await?;
	let h2 = harness.library.jobs().dispatch(job2).await?;
	let (r1, r2) = tokio::join!(h1.wait(), h2.wait());
	// At least one should succeed; the other may fail if deduped/cancelled by job system.
	assert!(
		r1.is_ok() || r2.is_ok(),
		"at least one concurrent reindex should succeed: r1={r1:?} r2={r2:?}"
	);

	let after = handle.get_all_entries().await?;
	let ids: HashSet<i32> = after.iter().map(|e| e.id).collect();
	assert_eq!(ids.len(), after.len(), "no duplicate entry IDs");
	let uuids: HashSet<Uuid> = after.iter().filter_map(|e| e.uuid).collect();
	assert_eq!(uuids.len(), after.len(), "no duplicate entry UUIDs");

	// Same file names still present; count should not grow.
	let names_before: HashSet<String> = before.iter().map(|e| e.name.clone()).collect();
	let names_after: HashSet<String> = after.iter().map(|e| e.name.clone()).collect();
	assert_eq!(
		names_before, names_after,
		"reindex should not change logical entry set"
	);

	harness.shutdown().await
}
