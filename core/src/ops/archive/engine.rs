//! Blocking archive reading and writing.
//!
//! Extraction treats every archive as hostile: entry names may not leave the
//! destination (zip slip), links may only point inside it, special files are
//! skipped, setuid bits are dropped, and nothing is written through a link.
//! Both directions poll a cancel flag and publish byte counts so the job can
//! report progress and stop promptly.

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use thiserror::Error;

use super::format::ArchiveFormat;

#[derive(Debug, Error)]
pub enum ArchiveError {
	#[error("{0}")]
	Io(#[from] io::Error),
	#[error("Zip error: {0}")]
	Zip(#[from] zip::result::ZipError),
	#[error("7-Zip error: {0}")]
	SevenZip(String),
	#[error("Unsafe archive entry rejected: {0}")]
	Unsafe(String),
	#[error("{0}")]
	Unsupported(String),
	#[error("Cancelled")]
	Cancelled,
}

/// Shared between the blocking worker and the job that watches it.
#[derive(Debug, Default)]
pub struct Control {
	pub cancel: AtomicBool,
	/// Bytes of input consumed so far
	pub done: AtomicU64,
	/// Entries written so far
	pub entries: AtomicU64,
}

impl Control {
	fn check(&self) -> Result<(), ArchiveError> {
		if self.cancel.load(Ordering::Relaxed) {
			Err(ArchiveError::Cancelled)
		} else {
			Ok(())
		}
	}
}

/// Reader that counts consumed bytes and fails once cancellation is requested.
struct Tracked<'a, R> {
	inner: R,
	control: &'a Control,
}

impl<R: Read> Read for Tracked<'_, R> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		if self.control.cancel.load(Ordering::Relaxed) {
			// Not `Interrupted`: io::copy retries that kind forever.
			return Err(io::Error::other("cancelled"));
		}
		let n = self.inner.read(buf)?;
		self.control.done.fetch_add(n as u64, Ordering::Relaxed);
		Ok(n)
	}
}

impl<R: io::Seek> io::Seek for Tracked<'_, R> {
	fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
		self.inner.seek(pos)
	}
}

fn cancelled_or(control: &Control, error: io::Error) -> ArchiveError {
	if control.cancel.load(Ordering::Relaxed) {
		ArchiveError::Cancelled
	} else {
		ArchiveError::Io(error)
	}
}

/// Validates an entry name, returning it relative to the destination.
pub fn safe_relative(name: &Path) -> Result<PathBuf, ArchiveError> {
	let normalized = PathBuf::from(name.to_string_lossy().replace('\\', "/"));
	let mut out = PathBuf::new();
	for component in normalized.components() {
		match component {
			Component::Normal(part) => out.push(part),
			Component::CurDir => {}
			_ => return Err(ArchiveError::Unsafe(name.display().to_string())),
		}
	}
	if out.as_os_str().is_empty() {
		return Err(ArchiveError::Unsafe(name.display().to_string()));
	}
	Ok(out)
}

/// A link at `link` (relative to the destination) may only point inside it.
fn check_link_target(link: &Path, target: &Path) -> Result<(), ArchiveError> {
	let unsafe_link =
		|| ArchiveError::Unsafe(format!("{} -> {}", link.display(), target.display()));
	if target.is_absolute() {
		return Err(unsafe_link());
	}
	let mut depth = link.components().count() as i64 - 1;
	for component in target.components() {
		match component {
			Component::Normal(_) => depth += 1,
			Component::CurDir => {}
			Component::ParentDir => {
				depth -= 1;
				if depth < 0 {
					return Err(unsafe_link());
				}
			}
			_ => return Err(unsafe_link()),
		}
	}
	Ok(())
}

/// Fails if any existing ancestor of `path` below `root` is a link, so an
/// entry can never be written through a link created earlier.
fn check_no_link_parents(root: &Path, relative: &Path) -> Result<(), ArchiveError> {
	let mut current = root.to_path_buf();
	let parents: Vec<_> = relative
		.parent()
		.into_iter()
		.flat_map(|p| p.components())
		.collect();
	for component in parents {
		current.push(component);
		if std::fs::symlink_metadata(&current).is_ok_and(|m| m.file_type().is_symlink()) {
			return Err(ArchiveError::Unsafe(relative.display().to_string()));
		}
	}
	Ok(())
}

/// Creates a regular file, replacing an earlier entry of the same name but
/// never following a link that sits there.
fn create_file(root: &Path, relative: &Path) -> Result<File, ArchiveError> {
	check_no_link_parents(root, relative)?;
	let path = root.join(relative);
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}
	if let Ok(meta) = std::fs::symlink_metadata(&path) {
		if meta.is_dir() {
			return Err(ArchiveError::Unsafe(relative.display().to_string()));
		}
		std::fs::remove_file(&path)?;
	}
	Ok(File::options().write(true).create_new(true).open(&path)?)
}

fn create_dir(root: &Path, relative: &Path) -> Result<(), ArchiveError> {
	check_no_link_parents(root, relative)?;
	std::fs::create_dir_all(root.join(relative))?;
	Ok(())
}

fn create_symlink(root: &Path, relative: &Path, target: &Path) -> Result<(), ArchiveError> {
	check_link_target(relative, target)?;
	check_no_link_parents(root, relative)?;
	let path = root.join(relative);
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}
	#[cfg(unix)]
	std::os::unix::fs::symlink(target, &path)?;
	#[cfg(not(unix))]
	return Err(ArchiveError::Unsupported(
		"Links in archives need a Unix system".into(),
	));
	#[cfg(unix)]
	Ok(())
}

#[cfg(unix)]
fn set_mode(file: &File, mode: u32) {
	use std::os::unix::fs::PermissionsExt;
	// Best-effort: the content is intact even if the bits cannot be applied.
	let _ = file.set_permissions(std::fs::Permissions::from_mode(mode & 0o777));
}

#[cfg(not(unix))]
fn set_mode(_file: &File, _mode: u32) {}

/// Extracts `archive` into the existing, empty directory `dest`.
pub fn extract(
	archive: &Path,
	format: ArchiveFormat,
	dest: &Path,
	control: &Control,
) -> Result<(), ArchiveError> {
	let file = File::open(archive)?;
	match format {
		ArchiveFormat::Zip => extract_zip(file, dest, control),
		ArchiveFormat::SevenZip => extract_7z(archive, dest, control),
		_ => {
			let tracked = Tracked {
				inner: io::BufReader::new(file),
				control,
			};
			let reader: Box<dyn Read + '_> = match format {
				ArchiveFormat::Tar => Box::new(tracked),
				ArchiveFormat::TarGz => Box::new(flate2::read::MultiGzDecoder::new(tracked)),
				ArchiveFormat::TarBz2 => Box::new(bzip2::read::MultiBzDecoder::new(tracked)),
				ArchiveFormat::TarXz => Box::new(xz2::read::XzDecoder::new_multi_decoder(tracked)),
				ArchiveFormat::TarZst => Box::new(zstd::stream::read::Decoder::new(tracked)?),
				ArchiveFormat::Zip | ArchiveFormat::SevenZip => unreachable!(),
			};
			extract_tar(reader, dest, control)
		}
	}
}

fn extract_zip(file: File, dest: &Path, control: &Control) -> Result<(), ArchiveError> {
	let mut zip = zip::ZipArchive::new(Tracked {
		inner: io::BufReader::new(file),
		control,
	})?;
	for index in 0..zip.len() {
		control.check()?;
		let mut entry = zip.by_index(index).map_err(|e| match e {
			zip::result::ZipError::Io(io) => cancelled_or(control, io),
			other => ArchiveError::Zip(other),
		})?;
		let relative = safe_relative(Path::new(entry.name()))?;
		if entry.is_dir() {
			create_dir(dest, &relative)?;
		} else if entry.is_symlink() {
			let mut target = String::new();
			entry.read_to_string(&mut target)?;
			create_symlink(dest, &relative, Path::new(&target))?;
		} else {
			let mut out = create_file(dest, &relative)?;
			io::copy(&mut entry, &mut out).map_err(|e| cancelled_or(control, e))?;
			if let Some(mode) = entry.unix_mode() {
				set_mode(&out, mode);
			}
		}
		control.entries.fetch_add(1, Ordering::Relaxed);
	}
	Ok(())
}

fn extract_tar(
	reader: Box<dyn Read + '_>,
	dest: &Path,
	control: &Control,
) -> Result<(), ArchiveError> {
	let mut archive = tar::Archive::new(reader);
	for entry in archive.entries().map_err(|e| cancelled_or(control, e))? {
		control.check()?;
		let mut entry = entry.map_err(|e| cancelled_or(control, e))?;
		let relative = safe_relative(&entry.path()?)?;
		let kind = entry.header().entry_type();
		if kind.is_dir() {
			create_dir(dest, &relative)?;
		} else if kind.is_symlink() {
			let target = entry
				.link_name()?
				.ok_or_else(|| ArchiveError::Unsafe(relative.display().to_string()))?;
			create_symlink(dest, &relative, &target)?;
		} else if kind.is_hard_link() {
			let target = entry
				.link_name()?
				.ok_or_else(|| ArchiveError::Unsafe(relative.display().to_string()))?;
			let target = safe_relative(&target)?;
			check_no_link_parents(dest, &target)?;
			let mut out = create_file(dest, &relative)?;
			io::copy(&mut File::open(dest.join(&target))?, &mut out)?;
		} else if kind.is_file() || kind == tar::EntryType::Continuous {
			let mut out = create_file(dest, &relative)?;
			io::copy(&mut entry, &mut out).map_err(|e| cancelled_or(control, e))?;
			if let Ok(mode) = entry.header().mode() {
				set_mode(&out, mode);
			}
		} else {
			// Devices, FIFOs and similar entries have no place in a user's folder.
			continue;
		}
		control.entries.fetch_add(1, Ordering::Relaxed);
	}
	Ok(())
}

fn extract_7z(archive: &Path, dest: &Path, control: &Control) -> Result<(), ArchiveError> {
	let mut failure: Option<ArchiveError> = None;
	let result =
		sevenz_rust2::decompress_file_with_extract_fn(archive, dest, |entry, reader, path| {
			let outcome = (|| {
				control.check()?;
				let relative = path
					.strip_prefix(dest)
					.map_err(|_| ArchiveError::Unsafe(entry.name().to_string()))?
					.to_path_buf();
				if entry.is_directory() {
					create_dir(dest, &relative)?;
				} else {
					let mut out = create_file(dest, &relative)?;
					let mut tracked = Tracked {
						inner: reader,
						control,
					};
					io::copy(&mut tracked, &mut out).map_err(|e| cancelled_or(control, e))?;
				}
				control.entries.fetch_add(1, Ordering::Relaxed);
				Ok::<(), ArchiveError>(())
			})();
			match outcome {
				Ok(()) => Ok(true),
				Err(error) => {
					let message = error.to_string();
					failure = Some(error);
					Err(sevenz_rust2::Error::Other(message.into()))
				}
			}
		});
	match (failure, result) {
		(Some(error), _) => Err(error),
		(None, Err(error)) => Err(ArchiveError::SevenZip(error.to_string())),
		(None, Ok(())) => Ok(()),
	}
}

/// One file system entry to archive, with its name inside the archive.
struct Item {
	path: PathBuf,
	name: PathBuf,
	meta: std::fs::Metadata,
}

/// Lists `sources` and their contents without following links. Each source
/// becomes a top-level entry named after its file name.
fn collect(sources: &[PathBuf], control: &Control) -> Result<(Vec<Item>, u64), ArchiveError> {
	let mut items = Vec::new();
	let mut total = 0;
	for source in sources {
		let name = PathBuf::from(source.file_name().ok_or_else(|| {
			ArchiveError::Unsupported(format!("Cannot archive {}", source.display()))
		})?);
		let mut stack = vec![(source.clone(), name)];
		while let Some((path, name)) = stack.pop() {
			control.check()?;
			let meta = std::fs::symlink_metadata(&path)?;
			if meta.is_dir() {
				let mut children: Vec<_> = std::fs::read_dir(&path)?
					.flatten()
					.map(|c| c.file_name())
					.collect();
				children.sort();
				for child in children.into_iter().rev() {
					stack.push((path.join(&child), name.join(&child)));
				}
			} else if meta.is_file() {
				total += meta.len();
			}
			items.push(Item { path, name, meta });
		}
	}
	Ok((items, total))
}

/// Total bytes of regular files under `sources`, for progress.
pub fn source_size(sources: &[PathBuf]) -> u64 {
	collect(sources, &Control::default())
		.map(|(_, total)| total)
		.unwrap_or(0)
}

fn archive_name(name: &Path) -> String {
	name.components()
		.map(|c| c.as_os_str().to_string_lossy())
		.collect::<Vec<_>>()
		.join("/")
}

#[cfg(unix)]
fn unix_mode(meta: &std::fs::Metadata) -> u32 {
	use std::os::unix::fs::PermissionsExt;
	meta.permissions().mode() & 0o777
}

#[cfg(not(unix))]
fn unix_mode(meta: &std::fs::Metadata) -> u32 {
	if meta.is_dir() {
		0o755
	} else {
		0o644
	}
}

/// Writes `sources` into a new archive at `output`, which must not exist.
pub fn compress(
	sources: &[PathBuf],
	format: ArchiveFormat,
	output: &Path,
	control: &Control,
) -> Result<(), ArchiveError> {
	if !format.can_write() {
		return Err(ArchiveError::Unsupported(
			"Creating 7-Zip archives is not supported".into(),
		));
	}
	let (items, _) = collect(sources, control)?;
	let file = File::options().write(true).create_new(true).open(output)?;
	let writer = io::BufWriter::new(file);
	match format {
		ArchiveFormat::Zip => compress_zip(&items, writer, control),
		ArchiveFormat::Tar => finish_tar(tar_builder(&items, writer, control)?),
		ArchiveFormat::TarGz => {
			let encoder = flate2::write::GzEncoder::new(writer, flate2::Compression::default());
			finish_tar(tar_builder(&items, encoder, control)?.finish()?)
		}
		ArchiveFormat::TarBz2 => {
			let encoder = bzip2::write::BzEncoder::new(writer, bzip2::Compression::default());
			finish_tar(tar_builder(&items, encoder, control)?.finish()?)
		}
		ArchiveFormat::TarXz => {
			let encoder = xz2::write::XzEncoder::new(writer, 6);
			finish_tar(tar_builder(&items, encoder, control)?.finish()?)
		}
		ArchiveFormat::TarZst => {
			let encoder = zstd::stream::write::Encoder::new(writer, 3)?;
			finish_tar(tar_builder(&items, encoder, control)?.finish()?)
		}
		ArchiveFormat::SevenZip => unreachable!(),
	}
}

fn finish_tar<W: Write>(mut writer: W) -> Result<(), ArchiveError> {
	writer.flush()?;
	Ok(())
}

fn tar_builder<W: Write>(items: &[Item], writer: W, control: &Control) -> Result<W, ArchiveError> {
	let mut builder = tar::Builder::new(writer);
	builder.follow_symlinks(false);
	for item in items {
		control.check()?;
		if item.meta.is_file() {
			let mut header = tar::Header::new_gnu();
			header.set_metadata(&item.meta);
			let reader = Tracked {
				inner: File::open(&item.path)?,
				control,
			};
			builder
				.append_data(&mut header, &item.name, reader)
				.map_err(|e| cancelled_or(control, e))?;
		} else {
			builder.append_path_with_name(&item.path, &item.name)?;
		}
		control.entries.fetch_add(1, Ordering::Relaxed);
	}
	Ok(builder.into_inner()?)
}

fn compress_zip<W: Write + io::Seek>(
	items: &[Item],
	writer: W,
	control: &Control,
) -> Result<(), ArchiveError> {
	use zip::write::SimpleFileOptions;
	let mut zip = zip::ZipWriter::new(writer);
	for item in items {
		control.check()?;
		let name = archive_name(&item.name);
		let options = SimpleFileOptions::default()
			.compression_method(zip::CompressionMethod::Deflated)
			.unix_permissions(unix_mode(&item.meta))
			.large_file(item.meta.len() >= u32::MAX as u64);
		if item.meta.is_dir() {
			zip.add_directory(format!("{name}/"), options)?;
		} else if item.meta.file_type().is_symlink() {
			let target = std::fs::read_link(&item.path)?;
			zip.add_symlink(name, target.to_string_lossy(), options)?;
		} else if item.meta.is_file() {
			zip.start_file(name, options)?;
			let mut reader = Tracked {
				inner: File::open(&item.path)?,
				control,
			};
			io::copy(&mut reader, &mut zip).map_err(|e| cancelled_or(control, e))?;
		}
		control.entries.fetch_add(1, Ordering::Relaxed);
	}
	zip.finish()?.flush()?;
	Ok(())
}

#[cfg(all(test, unix))]
mod tests {
	use super::*;

	fn tree(root: &Path) -> Vec<PathBuf> {
		let mut sources = Vec::new();
		let docs = root.join("docs");
		std::fs::create_dir_all(docs.join("deep")).unwrap();
		std::fs::write(docs.join("a.txt"), "alpha").unwrap();
		std::fs::write(docs.join("deep/b.bin"), vec![7u8; 300_000]).unwrap();
		std::os::unix::fs::symlink("a.txt", docs.join("link")).unwrap();
		let tool = root.join("tool.sh");
		std::fs::write(&tool, "#!/bin/sh\n").unwrap();
		use std::os::unix::fs::PermissionsExt;
		std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o755)).unwrap();
		sources.push(docs);
		sources.push(tool);
		sources
	}

	#[test]
	fn every_writable_format_round_trips() {
		for format in [
			ArchiveFormat::Zip,
			ArchiveFormat::Tar,
			ArchiveFormat::TarGz,
			ArchiveFormat::TarBz2,
			ArchiveFormat::TarXz,
			ArchiveFormat::TarZst,
		] {
			let dir = tempfile::tempdir().unwrap();
			let sources = tree(dir.path());
			let archive = dir.path().join(format!("out.{}", format.extension()));
			compress(&sources, format, &archive, &Control::default()).unwrap();
			assert_eq!(ArchiveFormat::detect(&archive), Some(format));

			let dest = dir.path().join("extracted");
			std::fs::create_dir(&dest).unwrap();
			let control = Control::default();
			extract(&archive, format, &dest, &control).unwrap();
			assert_eq!(
				std::fs::read_to_string(dest.join("docs/a.txt")).unwrap(),
				"alpha",
				"{format:?}"
			);
			assert_eq!(
				std::fs::read(dest.join("docs/deep/b.bin")).unwrap().len(),
				300_000
			);
			assert_eq!(
				std::fs::read_link(dest.join("docs/link")).unwrap(),
				Path::new("a.txt")
			);
			use std::os::unix::fs::PermissionsExt;
			let mode = std::fs::metadata(dest.join("tool.sh"))
				.unwrap()
				.permissions()
				.mode();
			assert_eq!(mode & 0o777, 0o755, "{format:?}");
			assert!(control.entries.load(Ordering::Relaxed) >= 5);
		}
	}

	#[test]
	fn seven_zip_archives_extract() {
		let dir = tempfile::tempdir().unwrap();
		let src = dir.path().join("src");
		std::fs::create_dir_all(src.join("inner")).unwrap();
		std::fs::write(src.join("inner/note.txt"), "seven").unwrap();
		let archive = dir.path().join("set.7z");
		sevenz_rust2::compress_to_path(&src, &archive).unwrap();

		let dest = dir.path().join("out");
		std::fs::create_dir(&dest).unwrap();
		extract(
			&archive,
			ArchiveFormat::SevenZip,
			&dest,
			&Control::default(),
		)
		.unwrap();
		let found = std::fs::read_to_string(dest.join("inner/note.txt"))
			.or_else(|_| std::fs::read_to_string(dest.join("src/inner/note.txt")))
			.unwrap();
		assert_eq!(found, "seven");
	}

	#[test]
	fn zip_slip_and_escaping_links_are_rejected() {
		let dir = tempfile::tempdir().unwrap();
		let archive = dir.path().join("evil.zip");
		{
			let mut zip = zip::ZipWriter::new(File::create(&archive).unwrap());
			let options = zip::write::SimpleFileOptions::default();
			zip.start_file("../escape.txt", options).unwrap();
			zip.write_all(b"pwned").unwrap();
			zip.finish().unwrap();
		}
		let dest = dir.path().join("out");
		std::fs::create_dir(&dest).unwrap();
		let error = extract(&archive, ArchiveFormat::Zip, &dest, &Control::default()).unwrap_err();
		assert!(matches!(error, ArchiveError::Unsafe(_)), "{error}");
		assert!(!dir.path().join("escape.txt").exists());

		let tarball = dir.path().join("evil.tar");
		{
			let mut builder = tar::Builder::new(File::create(&tarball).unwrap());
			let mut header = tar::Header::new_gnu();
			header.set_entry_type(tar::EntryType::Symlink);
			header.set_size(0);
			builder.append_link(&mut header, "etc", "/etc").unwrap();
			builder.finish().unwrap();
		}
		let dest = dir.path().join("out2");
		std::fs::create_dir(&dest).unwrap();
		let error = extract(&tarball, ArchiveFormat::Tar, &dest, &Control::default()).unwrap_err();
		assert!(matches!(error, ArchiveError::Unsafe(_)), "{error}");
	}

	#[test]
	fn writing_through_an_inner_link_is_rejected() {
		let dir = tempfile::tempdir().unwrap();
		let tarball = dir.path().join("sneaky.tar");
		{
			let mut builder = tar::Builder::new(File::create(&tarball).unwrap());
			let mut header = tar::Header::new_gnu();
			header.set_entry_type(tar::EntryType::Symlink);
			header.set_size(0);
			builder.append_link(&mut header, "sub", ".").unwrap();
			let mut file = tar::Header::new_gnu();
			file.set_size(1);
			file.set_mode(0o644);
			builder.append_data(&mut file, "sub/x", &b"x"[..]).unwrap();
			builder.finish().unwrap();
		}
		let dest = dir.path().join("out");
		std::fs::create_dir(&dest).unwrap();
		let error = extract(&tarball, ArchiveFormat::Tar, &dest, &Control::default()).unwrap_err();
		assert!(matches!(error, ArchiveError::Unsafe(_)), "{error}");
	}

	#[test]
	fn cancellation_mid_copy_returns_instead_of_spinning() {
		let control = Control::default();
		let mut reader = Tracked {
			inner: io::repeat(1),
			control: &control,
		};
		let mut sink = io::sink();
		control.cancel.store(true, Ordering::Relaxed);
		// io::copy retries ErrorKind::Interrupted; a cancel must surface as an error.
		assert!(io::copy(&mut reader, &mut sink).is_err());
	}

	#[test]
	fn cancellation_stops_compression() {
		let dir = tempfile::tempdir().unwrap();
		let sources = tree(dir.path());
		let control = Control::default();
		control.cancel.store(true, Ordering::Relaxed);
		let error = compress(
			&sources,
			ArchiveFormat::TarGz,
			&dir.path().join("x.tar.gz"),
			&control,
		)
		.unwrap_err();
		assert!(matches!(error, ArchiveError::Cancelled), "{error}");
	}

	#[test]
	fn link_targets_are_checked_lexically() {
		assert!(check_link_target(Path::new("a/link"), Path::new("../b")).is_ok());
		assert!(check_link_target(Path::new("a/link"), Path::new("../../b")).is_err());
		assert!(check_link_target(Path::new("link"), Path::new("/etc/passwd")).is_err());
		assert!(safe_relative(Path::new("a\\..\\..\\x")).is_err());
		assert!(safe_relative(Path::new("./a/b")).is_ok());
	}
}
