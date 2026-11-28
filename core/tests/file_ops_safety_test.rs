//! Targeted data-loss prevention tests for core file operations.
//!
//! Covers create, rename, and delete flows with temp fixtures to confirm
//! conflicts are refused, sources are preserved, and symlinks are not followed.

mod helpers;

use std::{path::Path, sync::OnceLock};

use helpers::*;
use tempfile::TempDir;
use tokio::fs;
use wing_core::{
	domain::addressing::{WingPath, WingPathBatch},
	infra::{action::LibraryAction, job::output::JobOutput},
	ops::files::{
		create_entry::{
			CreateFileAction, CreateFileInput, CreateSymlinkAction, CreateSymlinkInput,
		},
		create_folder::{CreateFolderAction, CreateFolderInput},
		delete::job::DeleteJob,
		rename::{FileRenameAction, FileRenameInput},
	},
};

fn trash_sandbox() -> &'static Path {
	static ROOT: OnceLock<TempDir> = OnceLock::new();
	ROOT.get_or_init(|| {
		let base = std::env::var_os("HOME")
			.map(std::path::PathBuf::from)
			.unwrap_or_else(std::env::temp_dir);
		let root = tempfile::Builder::new()
			.prefix("wing-cov03-trash-")
			.tempdir_in(base)
			.unwrap();
		std::env::set_var("XDG_DATA_HOME", root.path().join("data"));
		root
	})
	.path()
}

async fn write(path: &Path, content: &[u8]) {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent).await.unwrap();
	}
	fs::write(path, content).await.unwrap();
}

fn delete_outcome(output: &JobOutput) -> (usize, usize) {
	match output {
		JobOutput::FileDelete {
			deleted_count,
			failed_count,
			..
		} => (*deleted_count, *failed_count),
		other => panic!("unexpected output: {other:?}"),
	}
}

async fn dispatch_action<A>(harness: &IndexingHarness, input: A::Input) -> Result<A::Output, String>
where
	A: LibraryAction + Send + 'static,
	A::Input: Send,
	A::Output: Send + 'static,
{
	let action = A::from_input(input).map_err(|e| e.to_string())?;
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

#[tokio::test]
async fn create_file_creates_empty_file_and_refuses_existing() {
	let harness = IndexingHarnessBuilder::new("file_ops_create_file")
		.build()
		.await
		.unwrap();
	let loc = harness.create_test_location("loc").await.unwrap();
	let parent = WingPath::local(loc.path().to_path_buf());

	let out = dispatch_action::<CreateFileAction>(
		&harness,
		CreateFileInput {
			parent: parent.clone(),
			name: "notes.txt".to_string(),
		},
	)
	.await
	.unwrap();
	let created = out.path.as_local_path().unwrap();
	assert!(created.is_file());
	assert_eq!(fs::read(created).await.unwrap(), b"");

	let err = dispatch_action::<CreateFileAction>(
		&harness,
		CreateFileInput {
			parent,
			name: "notes.txt".to_string(),
		},
	)
	.await
	.unwrap_err();
	assert!(err.contains("already exists"), "{err}");
	assert_eq!(fs::read(created).await.unwrap(), b"");

	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn create_folder_creates_dir_and_refuses_invalid_name() {
	let harness = IndexingHarnessBuilder::new("file_ops_create_folder")
		.build()
		.await
		.unwrap();
	let loc = harness.create_test_location("loc").await.unwrap();
	let parent = WingPath::local(loc.path().to_path_buf());

	let out = dispatch_action::<CreateFolderAction>(
		&harness,
		CreateFolderInput::new(parent.clone(), "Projects"),
	)
	.await
	.unwrap();
	let created = out.folder_path.as_local_path().unwrap();
	assert!(created.is_dir());

	let err = dispatch_action::<CreateFolderAction>(
		&harness,
		CreateFolderInput::new(parent, "../escape"),
	)
	.await
	.unwrap_err();
	assert!(!err.is_empty(), "invalid name should be rejected");

	harness.shutdown().await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn create_symlink_creates_link_and_refuses_existing_name() {
	let harness = IndexingHarnessBuilder::new("file_ops_create_symlink")
		.build()
		.await
		.unwrap();
	let loc = harness.create_test_location("loc").await.unwrap();
	let root = loc.path();
	write(&root.join("target.txt"), b"payload").await;

	let parent = WingPath::local(root.to_path_buf());
	let out = dispatch_action::<CreateSymlinkAction>(
		&harness,
		CreateSymlinkInput {
			target: WingPath::local(root.join("target.txt")),
			parent: parent.clone(),
			name: "link.txt".to_string(),
		},
	)
	.await
	.unwrap();
	let link = out.path.as_local_path().unwrap();
	assert!(fs::symlink_metadata(link).await.unwrap().is_symlink());
	assert_eq!(fs::read(link).await.unwrap(), b"payload");

	let err = dispatch_action::<CreateSymlinkAction>(
		&harness,
		CreateSymlinkInput {
			target: WingPath::local(root.join("target.txt")),
			parent,
			name: "link.txt".to_string(),
		},
	)
	.await
	.unwrap_err();
	assert!(err.contains("already exists"), "{err}");
	assert_eq!(fs::read_link(link).await.unwrap(), root.join("target.txt"));

	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn delete_job_permanent_removes_files_and_empty_dirs() {
	let harness = IndexingHarnessBuilder::new("file_ops_delete_permanent")
		.build()
		.await
		.unwrap();
	let loc = harness.create_test_location("loc").await.unwrap();
	let root = loc.path();
	write(&root.join("a.txt"), b"a").await;
	write(&root.join("dir/b.txt"), b"b").await;

	let output = harness
		.library
		.jobs()
		.dispatch(DeleteJob::permanent(
			WingPathBatch::new(vec![
				WingPath::local(root.join("a.txt")),
				WingPath::local(root.join("dir")),
			]),
			true,
		))
		.await
		.unwrap()
		.wait()
		.await
		.unwrap();

	let (deleted, failed) = delete_outcome(&output);
	assert_eq!(deleted, 2, "failed={failed}");
	assert_eq!(failed, 0);
	assert!(!root.join("a.txt").exists());
	assert!(!root.join("dir").exists());

	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn delete_job_rejects_missing_target_without_touching_others() {
	let harness = IndexingHarnessBuilder::new("file_ops_delete_missing")
		.build()
		.await
		.unwrap();
	let loc = harness.create_test_location("loc").await.unwrap();
	let root = loc.path();
	write(&root.join("keep.txt"), b"keep").await;
	write(&root.join("remove.txt"), b"remove").await;

	let err = harness
		.library
		.jobs()
		.dispatch(DeleteJob::permanent(
			WingPathBatch::new(vec![
				WingPath::local(root.join("missing.txt")),
				WingPath::local(root.join("remove.txt")),
			]),
			true,
		))
		.await
		.unwrap()
		.wait()
		.await
		.unwrap_err();

	let msg = err.to_string();
	assert!(
		msg.contains("does not exist") || msg.contains("missing"),
		"unexpected error: {msg}"
	);
	assert_eq!(fs::read(root.join("keep.txt")).await.unwrap(), b"keep");
	assert_eq!(fs::read(root.join("remove.txt")).await.unwrap(), b"remove");

	harness.shutdown().await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn delete_job_permanent_removes_symlink_without_touching_target() {
	let harness = IndexingHarnessBuilder::new("file_ops_delete_symlink")
		.build()
		.await
		.unwrap();
	let loc = harness.create_test_location("loc").await.unwrap();
	let root = loc.path();
	write(&root.join("target.txt"), b"keep me").await;
	fs::symlink(root.join("target.txt"), root.join("link.txt"))
		.await
		.unwrap();

	let output = harness
		.library
		.jobs()
		.dispatch(DeleteJob::permanent(
			WingPathBatch::new(vec![WingPath::local(root.join("link.txt"))]),
			true,
		))
		.await
		.unwrap()
		.wait()
		.await
		.unwrap();

	let (deleted, failed) = delete_outcome(&output);
	assert_eq!((deleted, failed), (1, 0));
	assert!(fs::symlink_metadata(root.join("link.txt")).await.is_err());
	assert_eq!(fs::read(root.join("target.txt")).await.unwrap(), b"keep me");

	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn delete_job_trash_moves_file_out_of_place() {
	let root = trash_sandbox();
	let harness = IndexingHarnessBuilder::new("file_ops_delete_trash")
		.build()
		.await
		.unwrap();
	let file = root.join("trash-me.txt");
	write(&file, b"trashed").await;

	let output = harness
		.library
		.jobs()
		.dispatch(DeleteJob::trash(WingPathBatch::new(vec![WingPath::local(
			file.clone(),
		)])))
		.await
		.unwrap()
		.wait()
		.await
		.unwrap();

	let (deleted, failed) = delete_outcome(&output);
	assert_eq!((deleted, failed), (1, 0));
	assert!(!file.exists());

	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn rename_action_rejects_existing_target_and_preserves_source() {
	let harness = IndexingHarnessBuilder::new("file_ops_rename_conflict")
		.build()
		.await
		.unwrap();
	let loc = harness.create_test_location("loc").await.unwrap();
	let root = loc.path();
	write(&root.join("a.txt"), b"aaa").await;
	write(&root.join("b.txt"), b"bbb").await;

	let err = dispatch_action::<FileRenameAction>(
		&harness,
		FileRenameInput::new(WingPath::local(root.join("a.txt")), "b.txt"),
	)
	.await
	.unwrap_err();
	assert!(
		err.contains("already exists") || err.contains("exists"),
		"{err}"
	);
	assert_eq!(fs::read(root.join("a.txt")).await.unwrap(), b"aaa");
	assert_eq!(fs::read(root.join("b.txt")).await.unwrap(), b"bbb");

	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn rename_action_renames_file_and_preserves_content() {
	let harness = IndexingHarnessBuilder::new("file_ops_rename_ok")
		.build()
		.await
		.unwrap();
	let loc = harness.create_test_location("loc").await.unwrap();
	let root = loc.path();
	write(&root.join("old.txt"), b"content").await;

	let receipt = dispatch_action::<FileRenameAction>(
		&harness,
		FileRenameInput::new(WingPath::local(root.join("old.txt")), "new.txt"),
	)
	.await
	.unwrap();
	let _ = receipt;

	// Rename runs as a job; wait briefly for filesystem effect.
	let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
	while root.join("old.txt").exists() || !root.join("new.txt").exists() {
		assert!(
			std::time::Instant::now() < deadline,
			"rename did not complete"
		);
		tokio::time::sleep(std::time::Duration::from_millis(50)).await;
	}
	assert_eq!(fs::read(root.join("new.txt")).await.unwrap(), b"content");

	harness.shutdown().await.unwrap();
}
