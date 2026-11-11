//! Index verification: comparing a location's index with the filesystem.

mod helpers;

use helpers::*;
use wing_core::{
	infra::action::LibraryAction,
	location::IndexMode,
	ops::indexing::{IndexVerifyAction, IndexVerifyInput, IndexVerifyOutput},
};

async fn verify(
	harness: &IndexingHarness,
	path: &std::path::Path,
) -> anyhow::Result<IndexVerifyOutput> {
	let actions = harness.core.context.get_action_manager().await.unwrap();
	let action = IndexVerifyAction::from_input(IndexVerifyInput::new(path.to_path_buf()))
		.map_err(anyhow::Error::msg)?;
	Ok(actions
		.dispatch_library(Some(harness.library.id()), action)
		.await?)
}

fn issue_paths(
	issues: &[wing_core::ops::indexing::verify::output::IntegrityDifference],
) -> Vec<String> {
	let mut paths: Vec<_> = issues
		.iter()
		.map(|issue| {
			issue
				.path
				.file_name()
				.unwrap()
				.to_string_lossy()
				.into_owned()
		})
		.collect();
	paths.sort();
	paths
}

#[tokio::test]
async fn a_fresh_index_matches_the_filesystem() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("index_verify_clean")
		.disable_watcher()
		.build()
		.await?;
	let location = harness.create_test_location("tree").await?;
	location.write_file("a.txt", "alpha").await?;
	location.write_file("docs/b.txt", "bravo").await?;
	location.write_file("docs/deep/c.txt", "charlie").await?;
	location.index("Tree", IndexMode::Deep).await?;

	let output = verify(&harness, location.path()).await?;
	let report = &output.report;
	assert!(
		output.is_valid,
		"missing {:?}, stale {:?}, mismatched {:?}",
		report.missing_from_index, report.stale_in_index, report.metadata_mismatches
	);
	assert_eq!(report.filesystem_file_count, 3);
	assert_eq!(report.database_file_count, 3);
	assert_eq!(report.filesystem_dir_count, report.database_dir_count);
	assert_eq!(output.path, location.path());

	// A subfolder is verified on its own.
	let docs = verify(&harness, &location.path().join("docs")).await?;
	assert!(docs.is_valid, "{}", docs.report.summary);
	assert_eq!(docs.report.database_file_count, 2);

	// The verified folder itself is compared too: moving its modification
	// time behind the index is reported for that folder.
	let docs_path = location.path().join("docs");
	std::fs::File::open(&docs_path)?
		.set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(946_684_800))?;
	let drifted = verify(&harness, &docs_path).await?;
	assert!(!drifted.is_valid);
	let paths: Vec<_> = drifted
		.report
		.metadata_mismatches
		.iter()
		.map(|issue| issue.path.clone())
		.collect();
	assert_eq!(paths, [docs_path]);

	harness.shutdown().await
}

#[tokio::test]
async fn changes_made_behind_the_index_are_reported() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("index_verify_drift")
		.disable_watcher()
		.build()
		.await?;
	let location = harness.create_test_location("tree").await?;
	location.write_file("kept.txt", "same").await?;
	location.write_file("grown.txt", "short").await?;
	location.write_file("removed.txt", "gone soon").await?;
	location.index("Tree", IndexMode::Deep).await?;

	// The watcher is off, so the index does not see these changes.
	location.write_file("added.txt", "new").await?;
	location
		.write_file("grown.txt", "much longer than before")
		.await?;
	std::fs::remove_file(location.path().join("removed.txt"))?;

	let output = verify(&harness, location.path()).await?;
	let report = &output.report;
	assert!(!output.is_valid);
	assert_eq!(issue_paths(&report.missing_from_index), ["added.txt"]);
	assert_eq!(issue_paths(&report.stale_in_index), ["removed.txt"]);
	// The grown file, and possibly the folder whose entries changed.
	assert!(
		report
			.metadata_mismatches
			.iter()
			.all(|issue| issue.path.ends_with("grown.txt") || issue.path == location.path()),
		"{:?}",
		report.metadata_mismatches
	);
	assert!(!report.metadata_mismatches.is_empty());
	assert!(report.summary.contains('1'), "{}", report.summary);

	harness.shutdown().await
}

#[tokio::test]
async fn paths_outside_any_location_are_rejected() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("index_verify_outside")
		.disable_watcher()
		.build()
		.await?;
	let location = harness.create_test_location("tree").await?;
	location.write_file("a.txt", "alpha").await?;
	location.index("Tree", IndexMode::Deep).await?;

	let outside = harness.temp_path().join("elsewhere");
	std::fs::create_dir_all(&outside)?;
	let error = verify(&harness, &outside).await.unwrap_err();
	assert!(
		error
			.to_string()
			.contains("does not belong to any managed location"),
		"{error}"
	);

	let missing =
		IndexVerifyAction::from_input(IndexVerifyInput::new(harness.temp_path().join("missing")))
			.unwrap_err();
	assert!(missing.contains("Path does not exist"), "{missing}");

	harness.shutdown().await
}
