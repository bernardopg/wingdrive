//! A copy, move or rename job never replaces an existing item unless asked to.
//!
//! The copy action checks for conflicts before it queues the job, but the job
//! runs later: another job or app can create the same name in between.

mod helpers;

use std::path::Path;

use helpers::*;
use tokio::fs;
use wing_core::{
	domain::addressing::{WingPath, WingPathBatch},
	infra::job::output::JobOutput,
	ops::files::copy::{
		action::FileConflictResolution,
		input::CopyMethod,
		job::{CopyOptions, FileCopyJob},
	},
};

async fn write(path: &Path, content: &str) {
	fs::create_dir_all(path.parent().unwrap()).await.unwrap();
	fs::write(path, content).await.unwrap();
}

async fn run(harness: &IndexingHarness, job: FileCopyJob) -> JobOutput {
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

/// (done, failed) for a copy or move output.
fn counts(output: &JobOutput) -> (usize, usize) {
	match output {
		JobOutput::FileCopy {
			copied_count,
			failed_count,
			..
		} => (*copied_count, *failed_count),
		JobOutput::FileMove {
			moved_count,
			failed_count,
			..
		} => (*moved_count, *failed_count),
		other => panic!("unexpected output {other:?}"),
	}
}

fn copy_job(source: &Path, destination: &Path, options: CopyOptions) -> FileCopyJob {
	FileCopyJob::new(
		WingPathBatch::new(vec![WingPath::local(source.to_path_buf())]),
		WingPath::local(destination.to_path_buf()),
	)
	.with_options(options)
}

#[tokio::test]
async fn copy_without_a_policy_keeps_a_file_that_appeared_after_the_check() {
	let harness = IndexingHarnessBuilder::new("copy_conflict_no_policy")
		.build()
		.await
		.unwrap();
	let location = harness.create_test_location("conflict").await.unwrap();
	let root = location.path();
	write(&root.join("src/report.txt"), "new").await;
	write(&root.join("dest/report.txt"), "original").await;

	for method in [CopyMethod::Streaming, CopyMethod::Atomic] {
		let output = run(
			&harness,
			copy_job(
				&root.join("src/report.txt"),
				&root.join("dest"),
				CopyOptions {
					copy_method: method.clone(),
					..CopyOptions::default()
				},
			),
		)
		.await;
		assert_eq!(counts(&output), (0, 1), "{method:?}: {output:?}");
		assert_eq!(
			fs::read_to_string(root.join("dest/report.txt"))
				.await
				.unwrap(),
			"original",
			"{method:?}"
		);
	}
	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn copy_replaces_only_when_overwrite_is_chosen() {
	let harness = IndexingHarnessBuilder::new("copy_conflict_overwrite")
		.build()
		.await
		.unwrap();
	let location = harness.create_test_location("conflict").await.unwrap();
	let root = location.path();
	write(&root.join("src/a.txt"), "new a").await;
	write(&root.join("src/b.txt"), "new b").await;
	write(&root.join("dest/a.txt"), "old a").await;
	write(&root.join("dest/b.txt"), "old b").await;

	// The action sets this flag when the request asks to overwrite up front.
	let output = run(
		&harness,
		copy_job(
			&root.join("src/a.txt"),
			&root.join("dest"),
			CopyOptions {
				overwrite: true,
				..CopyOptions::default()
			},
		),
	)
	.await;
	assert_eq!(counts(&output), (1, 0));
	// And this policy when the user answers the conflict prompt.
	let output = run(
		&harness,
		copy_job(
			&root.join("src/b.txt"),
			&root.join("dest"),
			CopyOptions {
				overwrite: true,
				conflict_resolution: Some(FileConflictResolution::Overwrite),
				..CopyOptions::default()
			},
		),
	)
	.await;
	assert_eq!(counts(&output), (1, 0));

	assert_eq!(
		fs::read_to_string(root.join("dest/a.txt")).await.unwrap(),
		"new a"
	);
	assert_eq!(
		fs::read_to_string(root.join("dest/b.txt")).await.unwrap(),
		"new b"
	);
	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn rename_onto_an_existing_file_fails_and_keeps_both() {
	let harness = IndexingHarnessBuilder::new("rename_conflict_file")
		.build()
		.await
		.unwrap();
	let location = harness.create_test_location("conflict").await.unwrap();
	let root = location.path();
	write(&root.join("draft.txt"), "draft").await;
	write(&root.join("final.txt"), "final").await;

	let output = run(
		&harness,
		FileCopyJob::new_rename(WingPath::local(root.join("draft.txt")), "final.txt".into()),
	)
	.await;

	assert_eq!(counts(&output).1, 1, "{output:?}");
	assert_eq!(
		fs::read_to_string(root.join("draft.txt")).await.unwrap(),
		"draft"
	);
	assert_eq!(
		fs::read_to_string(root.join("final.txt")).await.unwrap(),
		"final"
	);
	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn rename_onto_an_existing_folder_does_not_move_into_it() {
	let harness = IndexingHarnessBuilder::new("rename_conflict_folder")
		.build()
		.await
		.unwrap();
	let location = harness.create_test_location("conflict").await.unwrap();
	let root = location.path();
	write(&root.join("notes.txt"), "notes").await;
	write(&root.join("docs/readme.txt"), "readme").await;

	let output = run(
		&harness,
		FileCopyJob::new_rename(WingPath::local(root.join("notes.txt")), "docs".into()),
	)
	.await;

	assert_eq!(counts(&output).1, 1, "{output:?}");
	assert_eq!(
		fs::read_to_string(root.join("notes.txt")).await.unwrap(),
		"notes"
	);
	assert!(fs::symlink_metadata(root.join("docs/notes.txt"))
		.await
		.is_err());
	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn rename_to_a_free_name_still_works() {
	let harness = IndexingHarnessBuilder::new("rename_free_name")
		.build()
		.await
		.unwrap();
	let location = harness.create_test_location("conflict").await.unwrap();
	let root = location.path();
	write(&root.join("old.txt"), "content").await;

	let output = run(
		&harness,
		FileCopyJob::new_rename(WingPath::local(root.join("old.txt")), "new.txt".into()),
	)
	.await;

	assert_eq!(counts(&output), (1, 0), "{output:?}");
	assert!(fs::symlink_metadata(root.join("old.txt")).await.is_err());
	assert_eq!(
		fs::read_to_string(root.join("new.txt")).await.unwrap(),
		"content"
	);
	harness.shutdown().await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn renaming_a_link_to_its_target_name_keeps_the_target() {
	let harness = IndexingHarnessBuilder::new("rename_link_onto_target")
		.build()
		.await
		.unwrap();
	let location = harness.create_test_location("conflict").await.unwrap();
	let root = location.path();
	write(&root.join("file.txt"), "data").await;
	std::os::unix::fs::symlink("file.txt", root.join("link")).unwrap();

	let output = run(
		&harness,
		FileCopyJob::new_rename(WingPath::local(root.join("link")), "file.txt".into()),
	)
	.await;

	assert_eq!(counts(&output).1, 1, "{output:?}");
	assert_eq!(
		fs::read_to_string(root.join("file.txt")).await.unwrap(),
		"data"
	);
	assert!(fs::symlink_metadata(root.join("file.txt"))
		.await
		.unwrap()
		.is_file());
	harness.shutdown().await.unwrap();
}
