//! Copying or moving a folder must keep the symbolic links inside it as links.
//!
//! Following them copies data from outside the selection, and a link that
//! points back at an ancestor makes the copy recurse into itself.

#![cfg(unix)]

mod helpers;

use std::os::unix::fs::symlink;
use std::path::Path;

use helpers::*;
use tokio::fs;
use wing_core::{
	domain::addressing::{WingPath, WingPathBatch},
	infra::job::output::JobOutput,
	ops::files::copy::{
		input::CopyMethod,
		job::{CopyOptions, FileCopyJob, MoveMode},
	},
};

async fn write(path: &Path, content: &str) {
	fs::create_dir_all(path.parent().unwrap()).await.unwrap();
	fs::write(path, content).await.unwrap();
}

/// A folder with a file, a link back to itself, a link to a folder outside the
/// selection and a dangling link.
async fn folder_with_links(root: &Path) -> std::path::PathBuf {
	let source = root.join("source");
	write(&source.join("inner/file.txt"), "inside").await;
	write(&root.join("outside/secret.txt"), "outside").await;
	symlink(".", source.join("self")).unwrap();
	symlink(root.join("outside"), source.join("outside-link")).unwrap();
	symlink("missing-target", source.join("dangling")).unwrap();
	source
}

async fn assert_links_preserved(copy: &Path) {
	assert_eq!(
		fs::read_to_string(copy.join("inner/file.txt"))
			.await
			.unwrap(),
		"inside"
	);
	for (name, target) in [("self", "."), ("dangling", "missing-target")] {
		let link = copy.join(name);
		assert!(
			fs::symlink_metadata(&link)
				.await
				.unwrap()
				.file_type()
				.is_symlink(),
			"{} is not a link",
			link.display()
		);
		assert_eq!(fs::read_link(&link).await.unwrap(), Path::new(target));
	}
	let outside = copy.join("outside-link");
	assert!(fs::symlink_metadata(&outside)
		.await
		.unwrap()
		.file_type()
		.is_symlink());
	// Nothing from outside the selection was copied as real data.
	assert!(!fs::symlink_metadata(copy.join("self/self"))
		.await
		.unwrap()
		.is_dir());
}

async fn run(
	harness: &IndexingHarness,
	source: &Path,
	destination: &Path,
	method: CopyMethod,
	move_mode: Option<MoveMode>,
) -> JobOutput {
	let job = FileCopyJob::new(
		WingPathBatch::new(vec![WingPath::local(source.to_path_buf())]),
		WingPath::local(destination.to_path_buf()),
	)
	.with_options(CopyOptions {
		copy_method: method,
		delete_after_copy: move_mode.is_some(),
		move_mode,
		..CopyOptions::default()
	});
	harness
		.library
		.jobs()
		.dispatch(job)
		.await
		.unwrap()
		.wait()
		.await
		.unwrap()
}

fn assert_no_failures(output: &JobOutput) {
	match output {
		JobOutput::FileCopy { failed_count, .. } | JobOutput::FileMove { failed_count, .. } => {
			assert_eq!(*failed_count, 0, "{output:?}")
		}
		other => panic!("unexpected output {other:?}"),
	}
}

#[tokio::test]
async fn streaming_copy_keeps_links_inside_a_folder() {
	let harness = IndexingHarnessBuilder::new("copy_links_streaming")
		.build()
		.await
		.unwrap();
	let location = harness.create_test_location("links").await.unwrap();
	let source = folder_with_links(location.path()).await;
	let destination = location.path().join("dest");
	fs::create_dir_all(&destination).await.unwrap();

	let output = run(&harness, &source, &destination, CopyMethod::Streaming, None).await;

	assert_no_failures(&output);
	assert_links_preserved(&destination.join("source")).await;
	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn atomic_copy_keeps_links_inside_a_folder() {
	let harness = IndexingHarnessBuilder::new("copy_links_atomic")
		.build()
		.await
		.unwrap();
	let location = harness.create_test_location("links").await.unwrap();
	let source = folder_with_links(location.path()).await;
	let destination = location.path().join("dest");
	fs::create_dir_all(&destination).await.unwrap();

	let output = run(&harness, &source, &destination, CopyMethod::Atomic, None).await;

	assert_no_failures(&output);
	assert_links_preserved(&destination.join("source")).await;
	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn streaming_move_keeps_links_and_leaves_their_targets() {
	let harness = IndexingHarnessBuilder::new("move_links_streaming")
		.build()
		.await
		.unwrap();
	let location = harness.create_test_location("links").await.unwrap();
	let source = folder_with_links(location.path()).await;
	let destination = location.path().join("dest");
	fs::create_dir_all(&destination).await.unwrap();

	let output = run(
		&harness,
		&source,
		&destination,
		CopyMethod::Streaming,
		Some(MoveMode::Move),
	)
	.await;

	assert_no_failures(&output);
	assert_links_preserved(&destination.join("source")).await;
	assert!(fs::symlink_metadata(&source).await.is_err());
	assert_eq!(
		fs::read_to_string(location.path().join("outside/secret.txt"))
			.await
			.unwrap(),
		"outside"
	);
	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn atomic_move_of_a_folder_with_a_link_to_itself_renames_it() {
	let harness = IndexingHarnessBuilder::new("move_links_atomic")
		.build()
		.await
		.unwrap();
	let location = harness.create_test_location("links").await.unwrap();
	let source = folder_with_links(location.path()).await;
	let destination = location.path().join("dest");
	fs::create_dir_all(&destination).await.unwrap();

	let output = run(
		&harness,
		&source,
		&destination,
		CopyMethod::Atomic,
		Some(MoveMode::Move),
	)
	.await;

	assert_no_failures(&output);
	assert_links_preserved(&destination.join("source")).await;
	assert!(fs::symlink_metadata(&source).await.is_err());
	harness.shutdown().await.unwrap();
}

/// The copy action counts the folder before it queues the job. Following links
/// there failed the whole copy on a dangling link before the job ran.
#[tokio::test]
async fn copy_action_queues_a_folder_with_a_dangling_link() {
	use wing_core::{
		infra::action::LibraryAction,
		ops::files::copy::{action::FileCopyAction, input::FileCopyInput},
	};
	let harness = IndexingHarnessBuilder::new("copy_action_links")
		.build()
		.await
		.unwrap();
	let location = harness.create_test_location("links").await.unwrap();
	let source = folder_with_links(location.path()).await;
	let destination = location.path().join("dest");
	fs::create_dir_all(&destination).await.unwrap();

	let action = <FileCopyAction as LibraryAction>::from_input(FileCopyInput::new(
		vec![source],
		&destination,
	))
	.unwrap();
	let receipt = {
		let manager = harness.core.context.action_manager.read().await;
		manager
			.as_ref()
			.unwrap()
			.dispatch_library(Some(harness.library.id()), action)
			.await
			.unwrap()
	};

	let copy = destination.join("source");
	let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
	while fs::symlink_metadata(copy.join("dangling")).await.is_err() {
		assert!(
			std::time::Instant::now() < deadline,
			"job {} never copied the folder",
			receipt.id
		);
		tokio::time::sleep(std::time::Duration::from_millis(100)).await;
	}
	assert_links_preserved(&copy).await;
	harness.shutdown().await.unwrap();
}

/// The delete job checks that its targets exist. Following a dangling link
/// reported it missing, so it could not be deleted.
#[tokio::test]
async fn delete_job_removes_a_selected_dangling_link() {
	use wing_core::ops::files::delete::job::DeleteJob;
	let harness = IndexingHarnessBuilder::new("delete_dangling_link")
		.build()
		.await
		.unwrap();
	let location = harness.create_test_location("links").await.unwrap();
	let link = location.path().join("dangling");
	symlink("missing-target", &link).unwrap();

	let output = harness
		.library
		.jobs()
		.dispatch(DeleteJob::permanent(
			WingPathBatch::new(vec![WingPath::local(link.clone())]),
			true,
		))
		.await
		.unwrap()
		.wait()
		.await;

	assert!(output.is_ok(), "{output:?}");
	assert!(fs::symlink_metadata(&link).await.is_err());
	harness.shutdown().await.unwrap();
}
