use std::path::Path;
use std::sync::atomic::Ordering;
use wing_core::ops::archive::{
	engine::{self, ArchiveError, Control},
	ArchiveFormat,
};

#[test]
fn archives_round_trip_and_extract_selected_members() {
	let root = tempfile::tempdir().unwrap();
	let source = root.path().join("source");
	std::fs::create_dir_all(source.join("nested")).unwrap();
	std::fs::write(source.join("nested/café.txt"), b"archive contents").unwrap();
	std::fs::write(source.join("other.txt"), b"other").unwrap();
	assert_eq!(engine::source_size(std::slice::from_ref(&source)), 21);

	for format in [
		ArchiveFormat::Zip,
		ArchiveFormat::Tar,
		ArchiveFormat::TarGz,
		ArchiveFormat::TarBz2,
		ArchiveFormat::TarXz,
		ArchiveFormat::TarZst,
	] {
		let archive = root.path().join(format!("test.{}", format.extension()));
		let output = root.path().join(format!("out-{}", format.extension()));
		std::fs::create_dir(&output).unwrap();
		engine::compress(
			std::slice::from_ref(&source),
			format,
			&archive,
			&Control::default(),
		)
		.unwrap();
		let entries = engine::list(&archive, format).unwrap();
		assert!(
			entries
				.iter()
				.any(|entry| entry.path == Path::new("source/nested/café.txt")),
			"{format:?}"
		);
		engine::extract_only(
			&archive,
			format,
			&output,
			Some(Path::new("source/nested")),
			&Control::default(),
		)
		.unwrap();
		assert_eq!(
			std::fs::read(output.join("source/nested/café.txt")).unwrap(),
			b"archive contents"
		);
		assert!(!output.join("source/other.txt").exists());
	}
}

#[test]
fn archives_refuse_unsafe_paths_and_cancelled_work() {
	assert!(engine::safe_relative(Path::new("../escape")).is_err());
	assert!(engine::safe_relative(Path::new("folder/../../escape")).is_err());
	let root = tempfile::tempdir().unwrap();
	let file = root.path().join("file.txt");
	std::fs::write(&file, b"data").unwrap();
	let control = Control::default();
	control.cancel.store(true, Ordering::Relaxed);
	let output = root.path().join("cancelled.zip");
	assert!(matches!(
		engine::compress(&[file], ArchiveFormat::Zip, &output, &control),
		Err(ArchiveError::Cancelled)
	));
	assert!(!output.exists());
}

#[test]
fn extraction_rejects_archive_entries_that_escape_the_destination() {
	use std::io::Write;

	let root = tempfile::tempdir().unwrap();
	let archive = root.path().join("unsafe.zip");
	let mut writer = zip::ZipWriter::new(std::fs::File::create(&archive).unwrap());
	writer
		.start_file("../escaped.txt", zip::write::SimpleFileOptions::default())
		.unwrap();
	writer.write_all(b"unsafe").unwrap();
	writer.finish().unwrap();
	let output = root.path().join("output");
	std::fs::create_dir(&output).unwrap();
	assert!(matches!(
		engine::extract(&archive, ArchiveFormat::Zip, &output, &Control::default()),
		Err(ArchiveError::Unsafe(_))
	));
	assert!(!root.path().join("escaped.txt").exists());
}
