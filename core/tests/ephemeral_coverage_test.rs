//! Complete ephemeral scans see every file, and browsing under a recursive
//! root reuses that root instead of scanning again (INDEX-011, INDEX-012).

mod helpers;

use std::path::{Path, PathBuf};

use anyhow::Result;
use helpers::IndexingHarnessBuilder;
use wing_core::{
	domain::addressing::WingPath,
	infra::{api::SessionContext, query::LibraryQuery},
	ops::{
		files::query::directory_listing::DirectoryListingQuery,
		indexing::{IndexScope, IndexerJob, IndexerJobConfig},
	},
};

/// A small git project with paths the default browsing rules drop.
async fn create_project(root: &Path) -> Result<()> {
	for dir in ["src", "node_modules/pkg", ".git/objects"] {
		tokio::fs::create_dir_all(root.join(dir)).await?;
	}
	for (file, body) in [
		("src/main.rs", "fn main() {}"),
		("node_modules/pkg/index.js", "module.exports = 1;"),
		(".git/HEAD", "ref: refs/heads/main"),
		(".env", "KEY=value"),
		("ignored.log", "log"),
		(".gitignore", "*.log\n"),
	] {
		tokio::fs::write(root.join(file), body).await?;
	}
	Ok(())
}

async fn run_scan(harness: &helpers::IndexingHarness, config: IndexerJobConfig) -> Result<()> {
	let path = config
		.path
		.as_local_path()
		.expect("local path")
		.to_path_buf();
	let scope = config.scope;
	let cache = harness.core.context.ephemeral_cache();
	let mut job = IndexerJob::new(config);
	job.set_ephemeral_index(cache.create_for_indexing(path, scope));
	harness.library.jobs().dispatch(job).await?.wait().await?;
	Ok(())
}

fn filtered_paths(root: &Path) -> [PathBuf; 4] {
	[
		root.join("node_modules/pkg/index.js"),
		root.join(".git/HEAD"),
		root.join(".gitignore"),
		root.join("ignored.log"),
	]
}

#[tokio::test]
async fn complete_scan_includes_paths_filtered_by_browse_rules() -> Result<()> {
	let harness = IndexingHarnessBuilder::new("complete_scan_filtered")
		.disable_watcher()
		.build()
		.await?;
	let root = harness.temp_path().join("project");
	create_project(&root).await?;

	run_scan(
		&harness,
		IndexerJobConfig::complete_scan(WingPath::local(root.clone()), IndexScope::Recursive),
	)
	.await?;

	let index = harness.core.context.ephemeral_cache().get_global_index();
	let index = index.read().await;
	for path in filtered_paths(&root)
		.into_iter()
		.chain([root.join("src/main.rs"), root.join(".env")])
	{
		assert!(index.has_entry(&path), "missing {}", path.display());
	}
	drop(index);
	harness.shutdown().await?;
	Ok(())
}

#[tokio::test]
async fn complete_scan_after_filtered_scan_fills_gaps_and_keeps_uuids() -> Result<()> {
	let harness = IndexingHarnessBuilder::new("complete_scan_additive")
		.disable_watcher()
		.build()
		.await?;
	let root = harness.temp_path().join("project");
	create_project(&root).await?;
	let wing_root = WingPath::local(root.clone());

	run_scan(
		&harness,
		IndexerJobConfig::ephemeral_browse(wing_root.clone(), IndexScope::Recursive, false),
	)
	.await?;

	let global = harness.core.context.ephemeral_cache().get_global_index();
	let main_rs = root.join("src/main.rs");
	let (before_uuid, before_len) = {
		let index = global.read().await;
		for path in filtered_paths(&root) {
			assert!(!index.has_entry(&path), "rules kept {}", path.display());
		}
		(
			index
				.get_entry_uuid(&main_rs)
				.expect("browse assigned a UUID"),
			index.len(),
		)
	};

	run_scan(
		&harness,
		IndexerJobConfig::complete_scan(wing_root, IndexScope::Recursive),
	)
	.await?;

	let index = global.read().await;
	assert_eq!(index.get_entry_uuid(&main_rs), Some(before_uuid));
	for path in filtered_paths(&root) {
		assert!(index.has_entry(&path), "gap not filled: {}", path.display());
	}
	// node_modules, node_modules/pkg, index.js, .git, .git/objects, HEAD,
	// .gitignore and ignored.log were filtered before; nothing else is new.
	assert_eq!(index.len(), before_len + 8);
	drop(index);
	assert_eq!(
		harness.core.context.ephemeral_cache().indexed_paths(),
		vec![root]
	);
	harness.shutdown().await?;
	Ok(())
}

#[tokio::test]
async fn browsing_under_recursive_root_reuses_it() -> Result<()> {
	let harness = IndexingHarnessBuilder::new("browse_under_recursive_root")
		.disable_watcher()
		.build()
		.await?;
	let root = harness.temp_path().join("volume");
	create_project(&root).await?;
	run_scan(
		&harness,
		IndexerJobConfig::ephemeral_browse(
			WingPath::local(root.clone()),
			IndexScope::Recursive,
			true,
		),
	)
	.await?;

	let session = SessionContext::device_session(
		harness.device_id,
		wing_core::device::get_current_device_slug(),
	)
	.with_library(harness.library.id());
	let listing = DirectoryListingQuery::new(WingPath::local(root.join("src")))
		.execute(harness.core.context.clone(), session)
		.await?;

	assert_eq!(listing.files.len(), 1);
	let cache = harness.core.context.ephemeral_cache();
	assert_eq!(cache.indexed_paths(), vec![root]);
	assert!(cache.paths_in_progress().is_empty());
	harness.shutdown().await?;
	Ok(())
}
