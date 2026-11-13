//! A copy or move that is cancelled or fails part-way loses no data.
//!
//! The source stays whole until its copy is complete, a destination that
//! existed before keeps its bytes until the new ones are in place, and no
//! half-written file is left under the destination name.

mod helpers;

use std::{
	path::{Path, PathBuf},
	time::{Duration, SystemTime},
};

use helpers::*;
use tokio::fs;
use wing_core::{
	domain::addressing::{WingPath, WingPathBatch},
	infra::job::{error::JobError, output::JobOutput},
	ops::files::copy::{
		input::CopyMethod,
		job::{CopyOptions, FileCopyJob, MoveMode},
	},
};

const METHODS: [CopyMethod; 2] = [CopyMethod::Streaming, CopyMethod::Atomic];

async fn write(path: &Path, content: &[u8]) {
	fs::create_dir_all(path.parent().unwrap()).await.unwrap();
	fs::write(path, content).await.unwrap();
}

async fn harness(name: &str) -> IndexingHarness {
	IndexingHarnessBuilder::new(name).build().await.unwrap()
}

fn job(sources: &[&Path], destination: &Path, options: CopyOptions) -> FileCopyJob {
	FileCopyJob::new(
		WingPathBatch::new(
			sources
				.iter()
				.map(|source| WingPath::local(source.to_path_buf()))
				.collect(),
		),
		WingPath::local(destination.to_path_buf()),
	)
	.with_options(options)
}

fn copy_with(method: CopyMethod) -> CopyOptions {
	CopyOptions {
		copy_method: method,
		..Default::default()
	}
}

fn move_with(method: CopyMethod) -> CopyOptions {
	CopyOptions {
		copy_method: method,
		delete_after_copy: true,
		move_mode: Some(MoveMode::Move),
		..Default::default()
	}
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

/// (done, failed, errors) for a copy or move output.
fn outcome(output: &JobOutput) -> (usize, usize, Vec<String>) {
	match output {
		JobOutput::FileCopy {
			copied_count,
			failed_count,
			errors,
			..
		} => (*copied_count, *failed_count, errors.clone()),
		JobOutput::FileMove {
			moved_count,
			failed_count,
			errors,
			..
		} => (*moved_count, *failed_count, errors.clone()),
		other => panic!("unexpected output {other:?}"),
	}
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

/// Makes a file unreadable. Returns `false` when that has no effect, as for root.
#[cfg(unix)]
async fn make_unreadable(path: &Path) -> bool {
	use std::os::unix::fs::PermissionsExt;
	fs::set_permissions(path, std::fs::Permissions::from_mode(0o000))
		.await
		.unwrap();
	std::fs::File::open(path).is_err()
}

#[tokio::test]
async fn empty_files_and_folders_are_copied_and_moved() {
	let harness = harness("copy_failure_empty").await;
	let location = harness.create_test_location("empty").await.unwrap();
	let root = location.path();

	for (index, method) in METHODS.into_iter().enumerate() {
		let source = root.join(format!("src{index}"));
		let destination = root.join(format!("dest{index}"));
		write(&source.join("empty.txt"), b"").await;
		fs::create_dir_all(source.join("hollow/inner"))
			.await
			.unwrap();
		fs::create_dir_all(&destination).await.unwrap();

		let output = run(
			&harness,
			job(
				&[&source.join("empty.txt"), &source.join("hollow")],
				&destination,
				copy_with(method.clone()),
			),
		)
		.await;
		assert_eq!(outcome(&output).1, 0, "{method:?}: {output:?}");
		assert_eq!(
			fs::metadata(destination.join("empty.txt"))
				.await
				.unwrap()
				.len(),
			0
		);
		assert!(destination.join("hollow/inner").is_dir(), "{method:?}");
		assert!(source.join("empty.txt").exists() && source.join("hollow/inner").is_dir());

		let moved = root.join(format!("moved{index}"));
		fs::create_dir_all(&moved).await.unwrap();
		let output = run(
			&harness,
			job(
				&[&source.join("empty.txt"), &source.join("hollow")],
				&moved,
				move_with(method.clone()),
			),
		)
		.await;
		assert_eq!(outcome(&output).1, 0, "{method:?}: {output:?}");
		assert!(moved.join("empty.txt").is_file() && moved.join("hollow/inner").is_dir());
		assert!(
			names(&source).await.is_empty(),
			"{method:?}: source not emptied"
		);
	}
}

#[tokio::test]
async fn copies_keep_content_and_modification_time() {
	let harness = harness("copy_failure_mtime").await;
	let location = harness.create_test_location("mtime").await.unwrap();
	let root = location.path();
	// Several 64 KiB chunks, with a tail that is not a whole chunk.
	let content: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
	let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1_500_000_000);

	let source = root.join("src");
	write(&source.join("album/track.bin"), &content).await;
	write(&source.join("album/notes.txt"), b"notes").await;
	for path in [
		source.join("album/track.bin"),
		source.join("album/notes.txt"),
		source.join("album"),
	] {
		std::fs::File::open(&path)
			.unwrap()
			.set_modified(old)
			.unwrap();
	}

	for (index, method) in METHODS.into_iter().enumerate() {
		let destination = root.join(format!("dest{index}"));
		fs::create_dir_all(&destination).await.unwrap();
		let mut options = copy_with(method.clone());
		options.verify_checksum = true;
		let output = run(
			&harness,
			job(
				&[&source.join("album"), &source.join("album/track.bin")],
				&destination,
				options,
			),
		)
		.await;
		assert_eq!(outcome(&output).1, 0, "{method:?}: {output:?}");

		for copy in [
			destination.join("track.bin"),
			destination.join("album/track.bin"),
		] {
			assert_eq!(fs::read(&copy).await.unwrap(), content, "{method:?}");
		}
		for copy in [
			destination.join("track.bin"),
			destination.join("album/track.bin"),
			destination.join("album/notes.txt"),
			destination.join("album"),
		] {
			assert_eq!(
				fs::metadata(&copy).await.unwrap().modified().unwrap(),
				old,
				"{method:?}: {} lost its modification time",
				copy.display()
			);
		}
	}
}

#[tokio::test]
async fn timestamps_are_new_when_the_option_is_off() {
	let harness = harness("copy_failure_no_mtime").await;
	let location = harness.create_test_location("nomtime").await.unwrap();
	let root = location.path();
	let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1_500_000_000);
	write(&root.join("src/a.txt"), b"a").await;
	std::fs::File::open(root.join("src/a.txt"))
		.unwrap()
		.set_modified(old)
		.unwrap();
	fs::create_dir_all(root.join("dest")).await.unwrap();

	let mut options = copy_with(CopyMethod::Streaming);
	options.preserve_timestamps = false;
	run(
		&harness,
		job(&[&root.join("src/a.txt")], &root.join("dest"), options),
	)
	.await;

	let copied = fs::metadata(root.join("dest/a.txt"))
		.await
		.unwrap()
		.modified()
		.unwrap();
	assert!(copied > old + Duration::from_secs(3600));
}

/// A copy that fails while reading leaves nothing under the destination.
/// `/proc/self/mem` opens fine and fails on the first read.
#[cfg(target_os = "linux")]
#[tokio::test]
async fn copy_that_fails_while_reading_leaves_no_partial_file() {
	let harness = harness("copy_failure_read_error").await;
	let location = harness.create_test_location("readerror").await.unwrap();
	let root = location.path();
	write(&root.join("dest/report.txt"), b"previous").await;

	let output = run(
		&harness,
		job(
			&[Path::new("/proc/self/mem")],
			&root.join("dest"),
			copy_with(CopyMethod::Streaming),
		),
	)
	.await;

	let (copied, failed, errors) = outcome(&output);
	assert_eq!((copied, failed), (0, 1), "{errors:?}");
	assert_eq!(names(&root.join("dest")).await, ["report.txt"]);
	assert_eq!(
		fs::read(root.join("dest/report.txt")).await.unwrap(),
		b"previous"
	);
}

/// A named pipe fed by a thread: a source that keeps a copy busy until the
/// test decides otherwise.
#[cfg(unix)]
struct SlowSource {
	path: PathBuf,
	chunks: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

#[cfg(unix)]
impl SlowSource {
	fn start(path: PathBuf) -> Self {
		use std::io::Write;
		use std::sync::atomic::Ordering;

		let status = std::process::Command::new("mkfifo")
			.arg(&path)
			.status()
			.unwrap();
		assert!(status.success());
		let chunks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
		let (writer_path, written) = (path.clone(), chunks.clone());
		std::thread::spawn(move || {
			// Blocks until the copy opens the pipe for reading.
			let Ok(mut pipe) = std::fs::OpenOptions::new().write(true).open(writer_path) else {
				return;
			};
			let chunk = vec![7u8; 256 * 1024];
			// Ends when the reader goes away.
			while pipe.write_all(&chunk).is_ok() {
				written.fetch_add(1, Ordering::SeqCst);
				std::thread::sleep(Duration::from_millis(5));
			}
		});
		Self { path, chunks }
	}

	/// Waits until the copy has read enough to be in the middle of the file.
	async fn wait_until_copying(&self) {
		use std::sync::atomic::Ordering;
		tokio::time::timeout(Duration::from_secs(20), async {
			// A pipe holds 64 KiB, so later chunks only fit after reads.
			while self.chunks.load(Ordering::SeqCst) < 8 {
				tokio::time::sleep(Duration::from_millis(10)).await;
			}
		})
		.await
		.expect("the copy never started reading");
	}
}

#[cfg(unix)]
#[tokio::test]
async fn cancelled_copy_leaves_no_partial_file() {
	let harness = harness("copy_failure_cancel").await;
	let location = harness.create_test_location("cancel").await.unwrap();
	let root = location.path();
	fs::create_dir_all(root.join("dest")).await.unwrap();
	let source = SlowSource::start(root.join("stream.bin"));

	let handle = harness
		.library
		.jobs()
		.dispatch(job(
			&[&source.path],
			&root.join("dest"),
			copy_with(CopyMethod::Streaming),
		))
		.await
		.unwrap();
	source.wait_until_copying().await;
	harness
		.library
		.jobs()
		.cancel_job(handle.id())
		.await
		.unwrap();

	let result = tokio::time::timeout(Duration::from_secs(20), handle.wait())
		.await
		.expect("the job did not stop");
	assert!(
		matches!(result, Err(JobError::Interrupted)),
		"a cancelled copy must end as cancelled, got {result:?}"
	);
	assert_eq!(
		names(&root.join("dest")).await,
		Vec::<String>::new(),
		"cancelling left files behind"
	);
}

#[cfg(unix)]
#[tokio::test]
async fn cancelled_overwrite_keeps_the_previous_file() {
	let harness = harness("copy_failure_cancel_overwrite").await;
	let location = harness.create_test_location("cancelow").await.unwrap();
	let root = location.path();
	let destination = root.join("dest/stream.bin");
	write(&destination, b"previous").await;
	let source = SlowSource::start(root.join("stream.bin"));

	let mut options = copy_with(CopyMethod::Streaming);
	options.overwrite = true;
	let handle = harness
		.library
		.jobs()
		.dispatch(job(&[&source.path], &destination, options))
		.await
		.unwrap();
	source.wait_until_copying().await;
	harness
		.library
		.jobs()
		.cancel_job(handle.id())
		.await
		.unwrap();
	let _ = tokio::time::timeout(Duration::from_secs(20), handle.wait())
		.await
		.expect("the job did not stop");

	assert_eq!(fs::read(&destination).await.unwrap(), b"previous");
	assert_eq!(names(&root.join("dest")).await, ["stream.bin"]);
}

#[cfg(unix)]
#[tokio::test]
async fn cancelled_move_keeps_the_source() {
	let harness = harness("copy_failure_cancel_move").await;
	let location = harness.create_test_location("cancelmove").await.unwrap();
	let root = location.path();
	fs::create_dir_all(root.join("dest")).await.unwrap();
	let source = SlowSource::start(root.join("stream.bin"));

	let handle = harness
		.library
		.jobs()
		.dispatch(job(
			&[&source.path],
			&root.join("dest"),
			move_with(CopyMethod::Streaming),
		))
		.await
		.unwrap();
	source.wait_until_copying().await;
	harness
		.library
		.jobs()
		.cancel_job(handle.id())
		.await
		.unwrap();
	let _ = tokio::time::timeout(Duration::from_secs(20), handle.wait())
		.await
		.expect("the job did not stop");

	assert!(fs::symlink_metadata(&source.path).await.is_ok());
	assert_eq!(names(&root.join("dest")).await, Vec::<String>::new());
}

/// Pipes, sockets and devices cannot be copied. Skipping one in silence made
/// a move delete the folder with the item still in it.
#[cfg(unix)]
#[tokio::test]
async fn folder_with_a_special_file_is_refused_and_kept() {
	let harness = harness("copy_failure_special").await;
	let location = harness.create_test_location("special").await.unwrap();
	let root = location.path();
	let folder = root.join("src/folder");
	write(&folder.join("first.txt"), b"first").await;
	let status = std::process::Command::new("mkfifo")
		.arg(folder.join("pipe"))
		.status()
		.unwrap();
	assert!(status.success());
	fs::create_dir_all(root.join("dest")).await.unwrap();

	let output = run(
		&harness,
		job(
			&[&folder],
			&root.join("dest"),
			move_with(CopyMethod::Streaming),
		),
	)
	.await;

	let (_, failed, errors) = outcome(&output);
	assert_eq!(failed, 1, "{errors:?}");
	assert!(errors[0].contains("not supported"), "{errors:?}");
	assert_eq!(names(&folder).await, ["first.txt", "pipe"]);
	assert_eq!(fs::read(folder.join("first.txt")).await.unwrap(), b"first");
}

#[cfg(unix)]
#[tokio::test]
async fn one_unreadable_file_does_not_stop_the_batch() {
	let harness = harness("copy_failure_partial_batch").await;
	let location = harness.create_test_location("partial").await.unwrap();
	let root = location.path();
	let source = root.join("src");
	for name in ["a.txt", "locked.txt", "z.txt"] {
		write(&source.join(name), name.as_bytes()).await;
	}
	if !make_unreadable(&source.join("locked.txt")).await {
		return; // root reads everything; nothing to test
	}
	let sources = [
		source.join("a.txt"),
		source.join("locked.txt"),
		source.join("z.txt"),
	];
	let sources: Vec<&Path> = sources.iter().map(PathBuf::as_path).collect();

	let copies = root.join("copies");
	fs::create_dir_all(&copies).await.unwrap();
	let output = run(
		&harness,
		job(&sources, &copies, copy_with(CopyMethod::Streaming)),
	)
	.await;
	let (copied, failed, errors) = outcome(&output);
	assert_eq!((copied, failed), (2, 1), "{errors:?}");
	assert!(errors[0].contains("locked.txt"), "{errors:?}");
	assert_eq!(names(&copies).await, ["a.txt", "z.txt"]);

	// Cross-volume moves copy and then delete; force that path on one volume.
	let moved = root.join("moved");
	fs::create_dir_all(&moved).await.unwrap();
	let output = run(
		&harness,
		job(&sources, &moved, move_with(CopyMethod::Streaming)),
	)
	.await;
	let (done, failed, errors) = outcome(&output);
	assert_eq!((done, failed), (2, 1), "{errors:?}");
	assert_eq!(names(&moved).await, ["a.txt", "z.txt"]);
	assert_eq!(
		names(&source).await,
		["locked.txt"],
		"only the file that could not be copied stays behind"
	);
}

#[cfg(unix)]
#[tokio::test]
async fn folder_move_that_fails_inside_keeps_the_whole_source_folder() {
	let harness = harness("copy_failure_partial_folder").await;
	let location = harness.create_test_location("partialdir").await.unwrap();
	let root = location.path();
	let folder = root.join("src/project");
	write(&folder.join("a.txt"), b"a").await;
	write(&folder.join("sub/locked.txt"), b"locked").await;
	write(&folder.join("z.txt"), b"z").await;
	if !make_unreadable(&folder.join("sub/locked.txt")).await {
		return;
	}
	fs::create_dir_all(root.join("dest")).await.unwrap();

	let output = run(
		&harness,
		job(
			&[&folder],
			&root.join("dest"),
			move_with(CopyMethod::Streaming),
		),
	)
	.await;

	let (_, failed, errors) = outcome(&output);
	assert_eq!(failed, 1, "{errors:?}");
	assert_eq!(names(&folder).await, ["a.txt", "sub", "z.txt"]);
	assert_eq!(fs::read(folder.join("a.txt")).await.unwrap(), b"a");
	assert_eq!(names(&folder.join("sub")).await, ["locked.txt"]);
	// No file under the destination is shorter than its source.
	let copied = root.join("dest/project");
	for name in ["a.txt", "z.txt"] {
		if copied.join(name).exists() {
			assert_eq!(
				fs::read(copied.join(name)).await.unwrap(),
				&name.as_bytes()[..1]
			);
		}
	}
	assert!(!copied.join("sub/locked.txt").exists());
}

/// The link the user selected is the item: it is copied, moved and renamed
/// as a link, whether or not its target exists.
#[cfg(unix)]
#[tokio::test]
async fn a_selected_link_is_copied_and_moved_as_a_link() {
	let harness = harness("copy_failure_selected_link").await;
	let location = harness.create_test_location("links").await.unwrap();
	let root = location.path();
	write(&root.join("src/target.txt"), b"target").await;
	fs::create_dir_all(root.join("src/folder")).await.unwrap();
	fs::symlink("target.txt", root.join("src/to-file"))
		.await
		.unwrap();
	fs::symlink("folder", root.join("src/to-folder"))
		.await
		.unwrap();
	fs::symlink("missing", root.join("src/dangling"))
		.await
		.unwrap();
	let links = ["to-file", "to-folder", "dangling"];
	let targets = ["target.txt", "folder", "missing"];

	for (index, method) in METHODS.into_iter().enumerate() {
		let copies = root.join(format!("copies{index}"));
		fs::create_dir_all(&copies).await.unwrap();
		let sources: Vec<PathBuf> = links.iter().map(|l| root.join("src").join(l)).collect();
		let sources: Vec<&Path> = sources.iter().map(PathBuf::as_path).collect();
		let output = run(&harness, job(&sources, &copies, copy_with(method.clone()))).await;
		let (_, failed, errors) = outcome(&output);
		assert_eq!(failed, 0, "{method:?}: {errors:?}");
		for (link, target) in links.iter().zip(targets) {
			assert_eq!(
				fs::read_link(copies.join(link)).await.unwrap(),
				Path::new(target),
				"{method:?}: {link} was not copied as a link"
			);
		}

		// Move the copies on, so the originals serve the next method too.
		let moved = root.join(format!("moved{index}"));
		fs::create_dir_all(&moved).await.unwrap();
		let sources: Vec<PathBuf> = links.iter().map(|l| copies.join(l)).collect();
		let sources: Vec<&Path> = sources.iter().map(PathBuf::as_path).collect();
		let output = run(&harness, job(&sources, &moved, move_with(method.clone()))).await;
		let (_, failed, errors) = outcome(&output);
		assert_eq!(failed, 0, "{method:?}: {errors:?}");
		for (link, target) in links.iter().zip(targets) {
			assert_eq!(
				fs::read_link(moved.join(link)).await.unwrap(),
				Path::new(target),
				"{method:?}: {link} was not moved as a link"
			);
		}
		assert!(names(&copies).await.is_empty(), "{method:?}");
	}

	// The targets were never touched.
	assert_eq!(
		fs::read(root.join("src/target.txt")).await.unwrap(),
		b"target"
	);
	assert!(root.join("src/folder").is_dir());

	// Renaming a dangling link renames the link.
	let output = run(
		&harness,
		FileCopyJob::new_rename(
			WingPath::local(root.join("src/dangling")),
			"still-dangling".to_string(),
		),
	)
	.await;
	let (_, failed, errors) = outcome(&output);
	assert_eq!(failed, 0, "{errors:?}");
	assert_eq!(
		fs::read_link(root.join("src/still-dangling"))
			.await
			.unwrap(),
		Path::new("missing")
	);
}

/// The action validates its sources before it queues a job, so a selected
/// dangling link has to pass there as well as in the job.
#[cfg(unix)]
#[tokio::test]
async fn copy_action_accepts_a_selected_dangling_link() {
	use wing_core::{
		infra::action::LibraryAction,
		ops::files::copy::{action::FileCopyAction, input::FileCopyInput},
	};
	let harness = harness("copy_failure_action_link").await;
	let location = harness.create_test_location("actionlink").await.unwrap();
	let root = location.path();
	fs::create_dir_all(root.join("dest")).await.unwrap();
	fs::symlink("missing", root.join("dangling")).await.unwrap();

	let action = <FileCopyAction as LibraryAction>::from_input(FileCopyInput::new(
		vec![root.join("dangling")],
		root.join("dest"),
	))
	.unwrap();
	{
		let manager = harness.core.context.action_manager.read().await;
		manager
			.as_ref()
			.unwrap()
			.dispatch_library(Some(harness.library.id()), action)
			.await
			.unwrap();
	}

	let copy = root.join("dest/dangling");
	let deadline = std::time::Instant::now() + Duration::from_secs(30);
	while fs::symlink_metadata(&copy).await.is_err() {
		assert!(
			std::time::Instant::now() < deadline,
			"the link was never copied"
		);
		tokio::time::sleep(Duration::from_millis(100)).await;
	}
	assert_eq!(fs::read_link(&copy).await.unwrap(), Path::new("missing"));
}
