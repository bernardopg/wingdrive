//! Folder totals after watcher changes (INDEX-014).
//!
//! `aggregate_size`, `file_count` and `child_count` of a directory used to be
//! computed only by a full indexing run, so a file created, grown, moved or
//! deleted afterwards left every folder above it showing stale totals. The
//! watcher now applies each change's difference to the ancestors; this test
//! drives each change kind through the watcher and checks the totals after it.

mod helpers;

use helpers::*;
use tokio::time::Duration;
use wing_core::{infra::db::entities, location::IndexMode};

/// Time for the watcher to debounce and apply a change.
const SETTLE: Duration = Duration::from_secs(6);

/// (aggregate_size, file_count, child_count) of the named directory.
fn totals(entries: &[entities::entry::Model], name: &str) -> (i64, i32, i32) {
	let dir = entries
		.iter()
		.find(|e| e.name == name && e.kind == 1)
		.unwrap_or_else(|| panic!("directory {name} should be indexed"));
	(dir.aggregate_size, dir.file_count, dir.child_count)
}

#[tokio::test]
async fn watcher_changes_keep_folder_totals_current() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("aggregate_sizes_watcher")
		.build()
		.await?;
	let test_location = harness.create_test_location("totals").await?;

	test_location.create_dir("alpha/inner").await?;
	test_location.create_dir("beta").await?;
	test_location
		.write_file("alpha/one.txt", "0123456789")
		.await?; // 10 B
	test_location
		.write_file("alpha/inner/two.txt", "01234567890123456789")
		.await?; // 20 B
	test_location.write_file("beta/three.txt", "01234").await?; // 5 B

	let location = test_location.index("Totals", IndexMode::Shallow).await?;
	tokio::time::sleep(Duration::from_millis(500)).await;

	let entries = location.get_all_entries().await?;
	assert_eq!(totals(&entries, "alpha"), (30, 2, 2));
	assert_eq!(totals(&entries, "inner"), (20, 1, 1));
	assert_eq!(totals(&entries, "beta"), (5, 1, 1));

	// Create: a new file counts in its folder and every folder above it.
	location
		.write_file("alpha/inner/new.txt", "0123456")
		.await?; // 7 B
	tokio::time::sleep(SETTLE).await;
	let entries = location.get_all_entries().await?;
	assert_eq!(totals(&entries, "inner"), (27, 2, 2), "after create");
	assert_eq!(totals(&entries, "alpha"), (37, 3, 2), "after create");

	// Resize: only the size difference is added.
	location
		.modify_file("beta/three.txt", "012345678901234")
		.await?; // 15 B
	tokio::time::sleep(SETTLE).await;
	let entries = location.get_all_entries().await?;
	assert_eq!(totals(&entries, "beta"), (15, 1, 1), "after resize");

	// Move: leaves the old folders and joins the new one.
	location
		.move_file("alpha/inner/new.txt", "beta/new.txt")
		.await?;
	tokio::time::sleep(SETTLE).await;
	let entries = location.get_all_entries().await?;
	assert_eq!(totals(&entries, "inner"), (20, 1, 1), "after move");
	assert_eq!(totals(&entries, "alpha"), (30, 2, 2), "after move");
	assert_eq!(totals(&entries, "beta"), (22, 2, 2), "after move");

	// Delete: the file's size and count leave every folder above it.
	location.delete_file("alpha/one.txt").await?;
	tokio::time::sleep(SETTLE).await;
	let entries = location.get_all_entries().await?;
	assert_eq!(totals(&entries, "alpha"), (20, 1, 1), "after delete");
	assert_eq!(totals(&entries, "inner"), (20, 1, 1), "after delete");

	location.verify_closure_table_integrity().await?;
	harness.shutdown().await?;
	Ok(())
}
