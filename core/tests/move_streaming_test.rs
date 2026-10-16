//! A move that copies instead of renaming must still remove its source.
//!
//! The streaming strategy copies bytes and leaves the source in place. The copy
//! job used to delete it only when the volume manager said the two paths were
//! on different volumes, so a streaming move inside one volume left the file in
//! both places and reported success.

use tempfile::TempDir;
use tokio::fs;
use wing_core::domain::addressing::{WingPath, WingPathBatch};
use wing_core::{
	infra::action::manager::ActionManager,
	ops::files::copy::{action::FileCopyAction, input::CopyMethod, job::CopyOptions},
	Core,
};

#[tokio::test]
async fn a_streaming_move_on_one_volume_removes_its_source() {
	let root = TempDir::new().unwrap();
	let source_dir = root.path().join("source");
	let dest_dir = root.path().join("dest");
	fs::create_dir_all(source_dir.join("folder")).await.unwrap();
	fs::create_dir_all(&dest_dir).await.unwrap();
	fs::write(source_dir.join("note.txt"), b"moved")
		.await
		.unwrap();
	fs::write(source_dir.join("folder/inner.txt"), b"inside")
		.await
		.unwrap();

	let core = Core::new(root.path().join("core_data")).await.unwrap();
	let library = core
		.libraries
		.create_library("Move Test Library", None, core.context.clone())
		.await
		.unwrap();

	let action = FileCopyAction {
		sources: WingPathBatch::new(vec![
			WingPath::local(source_dir.join("note.txt")),
			WingPath::local(source_dir.join("folder")),
		]),
		destination: WingPath::local(dest_dir.clone()),
		options: CopyOptions {
			conflict_resolution: None,
			overwrite: false,
			verify_checksum: false,
			preserve_timestamps: true,
			delete_after_copy: true,
			move_mode: None,
			copy_method: CopyMethod::Streaming,
		},
		on_conflict: None,
	};

	let receipt = ActionManager::new(core.context.clone())
		.dispatch_library(Some(library.id()), action)
		.await
		.expect("dispatch move");
	let handle = library
		.jobs()
		.get_job(receipt.id)
		.await
		.expect("move job is running");
	handle.wait().await.expect("move job completes");

	assert_eq!(fs::read(dest_dir.join("note.txt")).await.unwrap(), b"moved");
	assert_eq!(
		fs::read(dest_dir.join("folder/inner.txt")).await.unwrap(),
		b"inside"
	);
	assert!(
		!source_dir.join("note.txt").exists(),
		"moved file must not stay at its source"
	);
	assert!(
		!source_dir.join("folder").exists(),
		"moved folder must not stay at its source"
	);

	core.shutdown().await.unwrap();
}
