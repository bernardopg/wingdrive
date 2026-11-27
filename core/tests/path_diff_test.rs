//! `files.diff` compares two local folders through the ephemeral index.
//!
//! Each case writes real files, runs the action through the action manager, and checks what
//! it reports as missing, extra, modified and matched.

mod helpers;

use std::{
	path::{Path, PathBuf},
	time::{Duration, SystemTime},
};

use helpers::*;
use uuid::Uuid;
use wing_core::{
	domain::{addressing::WingPath, file::EntryKind},
	infra::action::LibraryAction,
	ops::files::diff::{DiffEntry, DiffStrategy, PathDiffAction, PathDiffInput, PathDiffResult},
};

async fn diff(
	harness: &IndexingHarness,
	source: WingPath,
	target: WingPath,
	strategy: DiffStrategy,
	use_index_rules: bool,
) -> Result<PathDiffResult, String> {
	let action = PathDiffAction::from_input(PathDiffInput {
		source,
		target,
		strategy,
		use_index_rules,
	})?;
	harness
		.core
		.context
		.action_manager
		.read()
		.await
		.as_ref()
		.unwrap()
		.dispatch_library(Some(harness.library.id()), action)
		.await
		.map_err(|e| e.to_string())
}

/// Writes `content` to `path` with a fixed modification time, so two copies compare equal.
fn write(path: &Path, content: &str, modified: SystemTime) {
	std::fs::write(path, content).unwrap();
	std::fs::File::options()
		.write(true)
		.open(path)
		.unwrap()
		.set_modified(modified)
		.unwrap();
}

/// Relative paths of the file entries in `entries`, sorted.
fn files(entries: &[DiffEntry]) -> Vec<PathBuf> {
	let mut paths: Vec<PathBuf> = entries
		.iter()
		.filter(|e| e.kind == EntryKind::File)
		.map(|e| e.relative_path.clone())
		.collect();
	paths.sort();
	paths
}

/// Two folders: `same.txt` in both, `changed.txt` with different sizes, one file only in each.
fn folders(root: &Path) -> (PathBuf, PathBuf) {
	let when = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
	let source = root.join("source");
	let target = root.join("target");
	std::fs::create_dir_all(&source).unwrap();
	std::fs::create_dir_all(&target).unwrap();
	write(&source.join("same.txt"), "identical", when);
	write(&target.join("same.txt"), "identical", when);
	write(&source.join("changed.txt"), "longer content", when);
	write(&target.join("changed.txt"), "short", when);
	write(&source.join("only-source.txt"), "12345", when);
	write(&target.join("only-target.txt"), "x", when);
	(source, target)
}

#[tokio::test]
async fn diff_reports_missing_extra_modified_and_matched_files() {
	let harness = IndexingHarnessBuilder::new("path_diff_strategies")
		.disable_watcher()
		.build()
		.await
		.unwrap();
	let (source, target) = folders(harness.temp_path());

	for (strategy, use_index_rules) in [
		(DiffStrategy::Heuristic, true),
		(DiffStrategy::Content, false),
		// Both folders are indexed by now, so this run reuses the ephemeral index.
		(DiffStrategy::Hybrid, true),
	] {
		let result = diff(
			&harness,
			WingPath::local(source.clone()),
			WingPath::local(target.clone()),
			strategy,
			use_index_rules,
		)
		.await
		.unwrap_or_else(|e| panic!("{strategy:?} diff failed: {e}"));
		assert_eq!(
			files(&result.only_in_source),
			[PathBuf::from("only-source.txt")],
			"{strategy:?}"
		);
		assert_eq!(
			files(&result.only_in_target),
			[PathBuf::from("only-target.txt")],
			"{strategy:?}"
		);
		assert_eq!(
			files(&result.modified),
			[PathBuf::from("changed.txt")],
			"{strategy:?}"
		);
		assert_eq!(result.matched_count, 1, "{strategy:?}");
		// What a sync from source to target would copy: the new file and the changed one.
		assert_eq!(
			result.copy_size,
			("12345".len() + "longer content".len()) as u64,
			"{strategy:?}"
		);
		assert_eq!(result.total_scanned, 3, "{strategy:?}");
		let entry = &result.only_in_source[0];
		assert_eq!(
			entry.wing_path.as_local_path(),
			Some(source.join("only-source.txt").as_path())
		);
		assert_eq!(entry.size, 5);
		assert_eq!(entry.modified_at, Some(1_700_000_000));
	}

	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn diff_refuses_paths_that_are_not_local_files() {
	let harness = IndexingHarnessBuilder::new("path_diff_refuses_content")
		.disable_watcher()
		.build()
		.await
		.unwrap();
	let local = WingPath::local(harness.temp_path().to_path_buf());

	let error = diff(
		&harness,
		WingPath::content(Uuid::new_v4()),
		local.clone(),
		DiffStrategy::Heuristic,
		true,
	)
	.await
	.unwrap_err();
	assert!(
		error.contains("Only local physical paths are supported for diff"),
		"{error}"
	);
	let error = diff(
		&harness,
		local,
		WingPath::content(Uuid::new_v4()),
		DiffStrategy::Heuristic,
		true,
	)
	.await
	.unwrap_err();
	assert!(error.contains("field 'target'"), "{error}");

	harness.shutdown().await.unwrap();
}
