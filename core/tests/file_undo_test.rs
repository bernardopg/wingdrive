//! Copy, move, rename and trash jobs journal what they changed, and
//! `files.undo` reverses exactly that.
//!
//! Undo refuses, item by item, anything that changed since the job, and
//! never overwrites or deletes data the job did not create.

mod helpers;

use std::path::{Path, PathBuf};

use helpers::*;
use tokio::fs;
use uuid::Uuid;
use wing_core::{
	domain::addressing::{WingPath, WingPathBatch},
	infra::job::{output::JobOutput, traits::JobHandler},
	ops::files::{
		copy::{
			action::FileConflictResolution,
			input::CopyMethod,
			job::{CopyOptions, FileCopyJob, MoveMode},
		},
		delete::job::DeleteJob,
		journal,
		undo::FileUndoJob,
	},
};

async fn write(path: &Path, content: &str) {
	fs::create_dir_all(path.parent().unwrap()).await.unwrap();
	fs::write(path, content).await.unwrap();
}

async fn read(path: &Path) -> String {
	fs::read_to_string(path).await.unwrap()
}

async fn harness(name: &str) -> IndexingHarness {
	IndexingHarnessBuilder::new(name).build().await.unwrap()
}

/// Names in a folder, sorted, including hidden ones.
async fn names(folder: &Path) -> Vec<String> {
	let mut names = Vec::new();
	let mut entries = fs::read_dir(folder).await.unwrap();
	while let Some(entry) = entries.next_entry().await.unwrap() {
		names.push(entry.file_name().to_string_lossy().into_owned());
	}
	names.sort();
	names
}

fn locals(paths: &[PathBuf]) -> WingPathBatch {
	WingPathBatch::new(paths.iter().cloned().map(WingPath::local).collect())
}

fn copy_job(sources: &[PathBuf], destination: &Path, options: CopyOptions) -> FileCopyJob {
	FileCopyJob::new(locals(sources), WingPath::local(destination.to_path_buf()))
		.with_options(options)
}

fn overwrite(method: CopyMethod) -> CopyOptions {
	CopyOptions {
		copy_method: method,
		conflict_resolution: Some(FileConflictResolution::Overwrite),
		..Default::default()
	}
}

fn moving(method: CopyMethod) -> CopyOptions {
	CopyOptions {
		copy_method: method,
		delete_after_copy: true,
		move_mode: Some(MoveMode::Move),
		..Default::default()
	}
}

/// Runs a job to its end and returns its id and output.
async fn run<J>(harness: &IndexingHarness, job: J) -> (Uuid, JobOutput)
where
	J: wing_core::infra::job::traits::Job + JobHandler + wing_core::infra::job::traits::DynJob,
{
	let handle = harness.library.jobs().dispatch(job).await.unwrap();
	let id = handle.id().0;
	(id, handle.wait().await.unwrap())
}

/// Undoes a job and returns (undone, skipped reasons).
async fn undo(harness: &IndexingHarness, job_id: Uuid) -> (usize, Vec<String>) {
	match run(harness, FileUndoJob { job_id }).await.1 {
		JobOutput::FileUndo {
			undone_count,
			skipped,
			..
		} => (undone_count, skipped),
		other => panic!("unexpected output {other:?}"),
	}
}

async fn pending(harness: &IndexingHarness, job_id: Uuid) -> usize {
	journal::pending(harness.library.jobs().database().conn(), job_id)
		.await
		.unwrap()
		.len()
}

#[tokio::test]
async fn undoing_a_copy_removes_only_what_it_created() {
	let harness = harness("undo_copy").await;
	let location = harness.create_test_location("copy").await.unwrap();
	let root = location.path();
	write(&root.join("src/a.txt"), "a").await;
	write(&root.join("src/album/one.txt"), "one").await;
	write(&root.join("src/album/inner/two.txt"), "two").await;
	write(&root.join("dest/keep.txt"), "keep").await;

	for method in [CopyMethod::Streaming, CopyMethod::Atomic] {
		let (job, _) = run(
			&harness,
			copy_job(
				&[root.join("src/a.txt"), root.join("src/album")],
				&root.join("dest"),
				CopyOptions {
					copy_method: method.clone(),
					..Default::default()
				},
			),
		)
		.await;
		assert_eq!(
			names(&root.join("dest")).await,
			["a.txt", "album", "keep.txt"]
		);

		assert_eq!(undo(&harness, job).await, (2, vec![]), "{method:?}");
		assert_eq!(names(&root.join("dest")).await, ["keep.txt"], "{method:?}");
		assert_eq!(pending(&harness, job).await, 0);
	}
	// The sources were never part of the undo.
	assert_eq!(read(&root.join("src/album/inner/two.txt")).await, "two");
	assert_eq!(read(&root.join("src/a.txt")).await, "a");
}

#[tokio::test]
async fn undoing_an_overwrite_brings_the_previous_file_back() {
	let harness = harness("undo_overwrite").await;
	let location = harness.create_test_location("overwrite").await.unwrap();
	let root = location.path();

	for method in [CopyMethod::Streaming, CopyMethod::Atomic] {
		write(&root.join("src/report.txt"), "new").await;
		write(&root.join("dest/report.txt"), "previous").await;

		let (job, _) = run(
			&harness,
			copy_job(
				&[root.join("src/report.txt")],
				&root.join("dest"),
				overwrite(method.clone()),
			),
		)
		.await;
		assert_eq!(read(&root.join("dest/report.txt")).await, "new");
		// The previous bytes are kept next to it until undo or expiry.
		let kept = names(&root.join("dest")).await;
		assert_eq!(kept.len(), 2, "{kept:?}");
		assert!(kept[0].starts_with(".wing-undo-"), "{kept:?}");

		assert_eq!(undo(&harness, job).await, (2, vec![]), "{method:?}");
		assert_eq!(read(&root.join("dest/report.txt")).await, "previous");
		assert_eq!(names(&root.join("dest")).await, ["report.txt"]);
	}
}

#[tokio::test]
async fn a_copy_changed_afterwards_is_left_alone() {
	let harness = harness("undo_changed").await;
	let location = harness.create_test_location("changed").await.unwrap();
	let root = location.path();
	write(&root.join("src/notes.txt"), "copied").await;
	write(&root.join("src/other.txt"), "other").await;
	write(&root.join("dest/notes.txt"), "previous").await;

	let (job, _) = run(
		&harness,
		copy_job(
			&[root.join("src/notes.txt"), root.join("src/other.txt")],
			&root.join("dest"),
			overwrite(CopyMethod::Streaming),
		),
	)
	.await;
	// The user keeps working on the copy before asking for undo.
	tokio::time::sleep(std::time::Duration::from_millis(20)).await;
	write(&root.join("dest/notes.txt"), "edited").await;

	let (undone, skipped) = undo(&harness, job).await;
	assert_eq!(undone, 1, "{skipped:?}");
	assert_eq!(skipped.len(), 2, "{skipped:?}");
	assert!(
		skipped[0].contains("changed after the operation"),
		"{skipped:?}"
	);
	assert!(skipped[1].contains("still there"), "{skipped:?}");

	// The edit survived, and the replaced file is still kept.
	assert_eq!(read(&root.join("dest/notes.txt")).await, "edited");
	assert!(!root.join("dest/other.txt").exists());
	assert_eq!(pending(&harness, job).await, 2);

	// Once the edited file is out of the way, the rest can be undone.
	fs::rename(root.join("dest/notes.txt"), root.join("edited.txt"))
		.await
		.unwrap();
	let (undone, skipped) = undo(&harness, job).await;
	assert_eq!(undone, 1, "{skipped:?}");
	assert_eq!(read(&root.join("dest/notes.txt")).await, "previous");
	assert_eq!(read(&root.join("edited.txt")).await, "edited");
}

#[tokio::test]
async fn undoing_a_move_puts_everything_back() {
	let harness = harness("undo_move").await;
	let location = harness.create_test_location("move").await.unwrap();
	let root = location.path();

	// Streaming copies and deletes, as a move across volumes does.
	for method in [CopyMethod::Atomic, CopyMethod::Streaming] {
		write(&root.join("src/a.txt"), "a").await;
		write(&root.join("src/album/one.txt"), "one").await;
		fs::create_dir_all(root.join("dest")).await.unwrap();

		let (job, _) = run(
			&harness,
			copy_job(
				&[root.join("src/a.txt"), root.join("src/album")],
				&root.join("dest"),
				moving(method.clone()),
			),
		)
		.await;
		assert!(names(&root.join("src")).await.is_empty(), "{method:?}");

		assert_eq!(undo(&harness, job).await, (2, vec![]), "{method:?}");
		assert_eq!(names(&root.join("src")).await, ["a.txt", "album"]);
		assert_eq!(read(&root.join("src/album/one.txt")).await, "one");
		assert!(names(&root.join("dest")).await.is_empty(), "{method:?}");
	}
}

#[tokio::test]
async fn undoing_a_move_never_overwrites_a_new_file_at_the_old_place() {
	let harness = harness("undo_move_taken").await;
	let location = harness.create_test_location("taken").await.unwrap();
	let root = location.path();
	write(&root.join("src/a.txt"), "moved").await;
	fs::create_dir_all(root.join("dest")).await.unwrap();

	let (job, _) = run(
		&harness,
		copy_job(
			&[root.join("src/a.txt")],
			&root.join("dest"),
			moving(CopyMethod::Atomic),
		),
	)
	.await;
	write(&root.join("src/a.txt"), "created later").await;

	let (undone, skipped) = undo(&harness, job).await;
	assert_eq!(undone, 0);
	assert!(skipped[0].contains("is taken again"), "{skipped:?}");
	assert_eq!(read(&root.join("src/a.txt")).await, "created later");
	assert_eq!(read(&root.join("dest/a.txt")).await, "moved");
}

#[tokio::test]
async fn undoing_a_rename_restores_the_old_name() {
	let harness = harness("undo_rename").await;
	let location = harness.create_test_location("rename").await.unwrap();
	let root = location.path();
	write(&root.join("draft.txt"), "text").await;

	let (job, _) = run(
		&harness,
		FileCopyJob::new_rename(WingPath::local(root.join("draft.txt")), "final.txt".into()),
	)
	.await;
	assert_eq!(names(root).await, ["final.txt"]);

	assert_eq!(undo(&harness, job).await, (1, vec![]));
	assert_eq!(names(root).await, ["draft.txt"]);
	assert_eq!(read(&root.join("draft.txt")).await, "text");
}

#[tokio::test]
async fn a_merge_into_an_existing_folder_is_reported_not_undone() {
	let harness = harness("undo_merge").await;
	let location = harness.create_test_location("merge").await.unwrap();
	let root = location.path();
	write(&root.join("src/album/new.txt"), "new").await;
	write(&root.join("dest/album/old.txt"), "old").await;

	let (job, _) = run(
		&harness,
		copy_job(
			&[root.join("src/album")],
			&root.join("dest"),
			overwrite(CopyMethod::Streaming),
		),
	)
	.await;
	assert_eq!(
		names(&root.join("dest/album")).await,
		["new.txt", "old.txt"]
	);

	let (undone, skipped) = undo(&harness, job).await;
	assert_eq!(undone, 0);
	assert!(skipped[0].contains("merged into a folder"), "{skipped:?}");
	assert_eq!(
		names(&root.join("dest/album")).await,
		["new.txt", "old.txt"]
	);
}

#[tokio::test]
async fn a_failed_overwrite_leaves_the_previous_file_and_nothing_to_undo() {
	let harness = harness("undo_failed_overwrite").await;
	let location = harness.create_test_location("failed").await.unwrap();
	let root = location.path();
	write(&root.join("dest/mem"), "previous").await;

	// Opens fine and fails on the first read.
	let source = if cfg!(target_os = "linux") {
		PathBuf::from("/proc/self/mem")
	} else {
		return;
	};
	let (job, output) = run(
		&harness,
		copy_job(
			&[source],
			&root.join("dest"),
			overwrite(CopyMethod::Streaming),
		),
	)
	.await;
	assert!(
		matches!(
			output,
			JobOutput::FileCopy {
				failed_count: 1,
				..
			}
		),
		"{output:?}"
	);

	assert_eq!(read(&root.join("dest/mem")).await, "previous");
	assert_eq!(names(&root.join("dest")).await, ["mem"]);
	assert_eq!(pending(&harness, job).await, 0);
}

#[tokio::test]
async fn expired_records_take_their_stashed_files_with_them() {
	let harness = harness("undo_expiry").await;
	let location = harness.create_test_location("expiry").await.unwrap();
	let root = location.path();
	write(&root.join("src/report.txt"), "new").await;
	write(&root.join("dest/report.txt"), "previous").await;

	let (job, _) = run(
		&harness,
		copy_job(
			&[root.join("src/report.txt")],
			&root.join("dest"),
			overwrite(CopyMethod::Streaming),
		),
	)
	.await;
	let db = harness.library.jobs().database().conn();

	// Nothing is old enough yet.
	let week_ago = chrono::Utc::now() - chrono::Duration::days(journal::RETENTION_DAYS);
	assert_eq!(journal::purge_expired(db, week_ago).await, 0);
	assert_eq!(pending(&harness, job).await, 2);

	let later = chrono::Utc::now() + chrono::Duration::seconds(1);
	assert_eq!(journal::purge_expired(db, later).await, 2);
	assert_eq!(names(&root.join("dest")).await, ["report.txt"]);
	assert_eq!(read(&root.join("dest/report.txt")).await, "new");
	assert_eq!(pending(&harness, job).await, 0);
}

#[tokio::test]
async fn history_lists_operations_newest_first_with_what_is_left() {
	let harness = harness("undo_history").await;
	let location = harness.create_test_location("history").await.unwrap();
	let root = location.path();
	write(&root.join("src/a.txt"), "a").await;
	write(&root.join("src/b.txt"), "b").await;
	fs::create_dir_all(root.join("dest")).await.unwrap();

	let (copy, _) = run(
		&harness,
		copy_job(
			&[root.join("src/a.txt"), root.join("src/b.txt")],
			&root.join("dest"),
			CopyOptions::default(),
		),
	)
	.await;
	let (rename, _) = run(
		&harness,
		FileCopyJob::new_rename(WingPath::local(root.join("src/a.txt")), "z.txt".into()),
	)
	.await;

	let db = harness.library.jobs().database().conn();
	let history = journal::history(db, 10).await.unwrap();
	assert_eq!(
		history
			.iter()
			.map(|op| (op.job_id, op.operation.as_str(), op.effects, op.pending))
			.collect::<Vec<_>>(),
		[(rename, "rename", 1, 1), (copy, "copy", 2, 2)]
	);
	assert_eq!(history[0].example, root.join("src/z.txt"));

	undo(&harness, copy).await;
	let history = journal::history(db, 10).await.unwrap();
	assert_eq!((history[1].effects, history[1].pending), (2, 0));
	assert_eq!(journal::history(db, 1).await.unwrap().len(), 1);
}

/// The home trash follows `XDG_DATA_HOME`; this is the only test in the
/// binary that reads it, and the trash stays on the same mount as the files.
#[cfg(target_os = "linux")]
#[tokio::test]
async fn undoing_a_trash_restores_the_items() {
	let harness = harness("undo_trash").await;
	let location = harness.create_test_location("trash").await.unwrap();
	let root = location.path();
	std::env::set_var("XDG_DATA_HOME", root.join("xdg"));
	write(&root.join("files/a.txt"), "a").await;
	write(&root.join("files/album/one.txt"), "one").await;
	write(&root.join("files/gone.txt"), "gone").await;
	let targets = [
		root.join("files/a.txt"),
		root.join("files/album"),
		root.join("files/gone.txt"),
	];

	let (job, _) = run(&harness, DeleteJob::trash(locals(&targets))).await;
	assert!(names(&root.join("files")).await.is_empty());
	// A new file took the name of one of them in the meantime.
	write(&root.join("files/gone.txt"), "created later").await;

	let (undone, skipped) = undo(&harness, job).await;
	assert_eq!(undone, 2, "{skipped:?}");
	assert_eq!(skipped.len(), 1);
	assert!(
		skipped[0].contains("something else is at its place"),
		"{skipped:?}"
	);
	assert_eq!(read(&root.join("files/a.txt")).await, "a");
	assert_eq!(read(&root.join("files/album/one.txt")).await, "one");
	assert_eq!(read(&root.join("files/gone.txt")).await, "created later");

	// A permanent delete records nothing: there is nothing to bring back.
	let (job, _) = run(
		&harness,
		DeleteJob::permanent(locals(&[root.join("files/a.txt")]), true),
	)
	.await;
	assert_eq!(pending(&harness, job).await, 0);
}

#[tokio::test]
async fn the_undo_action_picks_the_latest_operation_and_refuses_when_empty() {
	use wing_core::{
		infra::action::LibraryAction,
		ops::files::undo::{FileUndoAction, FileUndoInput},
	};
	let harness = harness("undo_action").await;
	let location = harness.create_test_location("action").await.unwrap();
	let root = location.path();
	write(&root.join("src/a.txt"), "a").await;
	fs::create_dir_all(root.join("dest")).await.unwrap();

	let dispatch = || async {
		let action =
			<FileUndoAction as LibraryAction>::from_input(FileUndoInput { job_id: None }).unwrap();
		let manager = harness.core.context.action_manager.read().await;
		manager
			.as_ref()
			.unwrap()
			.dispatch_library(Some(harness.library.id()), action)
			.await
	};
	let error = dispatch().await.unwrap_err();
	assert!(error.to_string().contains("Nothing to undo"), "{error}");

	run(
		&harness,
		copy_job(
			&[root.join("src/a.txt")],
			&root.join("dest"),
			CopyOptions::default(),
		),
	)
	.await;
	let receipt = dispatch().await.unwrap();
	assert_eq!(receipt.job_name, "file_undo");

	let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
	while root.join("dest/a.txt").exists() {
		assert!(
			std::time::Instant::now() < deadline,
			"the copy was never undone"
		);
		tokio::time::sleep(std::time::Duration::from_millis(50)).await;
	}
	assert_eq!(read(&root.join("src/a.txt")).await, "a");
}
