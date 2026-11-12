//! # Copies and moves that involve a cloud volume
//!
//! The local copy strategies work on filesystem paths. When a source or the
//! destination is a `WingPath::Cloud`, `FileCopyJob` hands the whole batch to
//! this module, which moves bytes through `VolumeBackend`: uploads stream from
//! the local file, downloads write through a temporary sibling, and a copy
//! between two cloud volumes goes through a temporary file under the system
//! temp folder. A move inside one cloud volume uses the service's own rename,
//! which is instant even for folders, and a copy inside one volume uses its
//! server-side copy when the service has one.
//!
//! Conflicts follow the job's `FileConflictResolution`, the same as local
//! copies. A move deletes each source only after its copy succeeded.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use super::action::FileConflictResolution;
use super::job::{CopyError, CopyPhase, CopyProgress, FileCopyOutput, MoveMode};
use crate::domain::addressing::WingPath;
use crate::infra::job::prelude::*;
use crate::ops::indexing::state::EntryKind;
use crate::volume::backend::VolumeBackend;
use crate::volume::error::VolumeError;
use crate::volume::VolumeManager;

/// One side of a transfer.
#[derive(Clone)]
pub enum Endpoint {
	Local(PathBuf),
	Cloud {
		backend: Arc<dyn VolumeBackend>,
		path: PathBuf,
		/// "scheme://identifier", equal for two paths on the same volume.
		volume: String,
	},
}

impl std::fmt::Debug for Endpoint {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.display())
	}
}

impl Endpoint {
	/// Resolves a `WingPath` on this device or on a mounted cloud volume.
	pub async fn resolve(path: &WingPath, volumes: Option<&VolumeManager>) -> Result<Self, String> {
		if let Some(local) = path.as_local_path() {
			return Ok(Self::Local(local.to_path_buf()));
		}
		let (service, identifier, cloud_path) = path
			.as_cloud()
			.ok_or_else(|| format!("{} is on another device", path.display()))?;
		let volumes = volumes.ok_or("Volume manager not available")?;
		let volume = volumes
			.find_cloud_volume(service, identifier)
			.await
			.ok_or_else(|| {
				format!(
					"Cloud volume {}://{} is not connected",
					service.scheme(),
					identifier
				)
			})?;
		let backend = volume
			.backend
			.clone()
			.ok_or_else(|| "Cloud volume has no backend".to_string())?;
		Ok(Self::Cloud {
			backend,
			path: PathBuf::from(cloud_path.trim_matches('/')),
			volume: format!("{}://{}", service.scheme(), identifier),
		})
	}

	pub fn display(&self) -> String {
		match self {
			Self::Local(path) => path.display().to_string(),
			Self::Cloud { path, volume, .. } => format!("{volume}/{}", path.display()),
		}
	}

	fn path(&self) -> &Path {
		match self {
			Self::Local(path) | Self::Cloud { path, .. } => path,
		}
	}

	fn name(&self) -> Option<String> {
		self.path()
			.file_name()
			.map(|n| n.to_string_lossy().into_owned())
	}

	pub fn join(&self, name: &str) -> Self {
		match self {
			Self::Local(path) => Self::Local(path.join(name)),
			Self::Cloud {
				backend,
				path,
				volume,
			} => Self::Cloud {
				backend: backend.clone(),
				path: path.join(name),
				volume: volume.clone(),
			},
		}
	}

	fn with_name(&self, name: &str) -> Self {
		match self {
			Self::Local(path) => Self::Local(path.with_file_name(name)),
			Self::Cloud {
				backend,
				path,
				volume,
			} => Self::Cloud {
				backend: backend.clone(),
				path: path.with_file_name(name),
				volume: volume.clone(),
			},
		}
	}

	fn same_volume(&self, other: &Self) -> bool {
		matches!((self, other), (Self::Cloud { volume: a, .. }, Self::Cloud { volume: b, .. }) if a == b)
	}

	/// The kind of entry at this path, or `None` when nothing is there.
	pub async fn kind(&self) -> Result<Option<EntryKind>, VolumeError> {
		match self {
			Self::Local(path) => match tokio::fs::symlink_metadata(path).await {
				Ok(m) if m.is_dir() => Ok(Some(EntryKind::Directory)),
				Ok(m) if m.file_type().is_symlink() => Ok(Some(EntryKind::Symlink)),
				Ok(_) => Ok(Some(EntryKind::File)),
				Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
				Err(e) => Err(e.into()),
			},
			Self::Cloud { backend, path, .. } => {
				if path.as_os_str().is_empty() {
					return Ok(Some(EntryKind::Directory));
				}
				match backend.metadata(path).await {
					Ok(m) => Ok(Some(m.kind)),
					Err(VolumeError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
					Err(e) => Err(e),
				}
			}
		}
	}

	async fn children(&self) -> Result<Vec<String>, VolumeError> {
		match self {
			Self::Local(path) => {
				let mut names = Vec::new();
				let mut entries = tokio::fs::read_dir(path).await?;
				while let Some(entry) = entries.next_entry().await? {
					names.push(entry.file_name().to_string_lossy().into_owned());
				}
				Ok(names)
			}
			Self::Cloud { backend, path, .. } => Ok(backend
				.read_dir(path)
				.await?
				.into_iter()
				.map(|e| e.name)
				.collect()),
		}
	}

	async fn create_dir(&self) -> Result<(), VolumeError> {
		match self {
			Self::Local(path) => Ok(tokio::fs::create_dir_all(path).await?),
			Self::Cloud { backend, path, .. } => backend.create_directory(path, true).await,
		}
	}

	async fn remove(&self, kind: EntryKind) -> Result<(), VolumeError> {
		match self {
			Self::Local(path) if kind == EntryKind::Directory => {
				Ok(tokio::fs::remove_dir_all(path).await?)
			}
			Self::Local(path) => Ok(tokio::fs::remove_file(path).await?),
			Self::Cloud { backend, path, .. } => backend.delete(path).await,
		}
	}
}

/// Copies one file between endpoints and returns the bytes moved.
pub async fn transfer_file(source: &Endpoint, destination: &Endpoint) -> Result<u64, VolumeError> {
	match (source, destination) {
		(Endpoint::Local(from), Endpoint::Local(to)) => Ok(tokio::fs::copy(from, to).await?),
		(Endpoint::Local(from), Endpoint::Cloud { backend, path, .. }) => {
			backend.write_from_file(path, from).await
		}
		(Endpoint::Cloud { backend, path, .. }, Endpoint::Local(to)) => {
			if let Some(parent) = to.parent() {
				tokio::fs::create_dir_all(parent).await?;
			}
			backend.read_to_file(path, to).await
		}
		(
			Endpoint::Cloud {
				backend: from_backend,
				path: from,
				..
			},
			Endpoint::Cloud {
				backend: to_backend,
				path: to,
				..
			},
		) => {
			if source.same_volume(destination) {
				match from_backend.copy(from, to).await {
					Ok(()) => {
						return Ok(from_backend.metadata(to).await.map(|m| m.size).unwrap_or(0))
					}
					Err(VolumeError::Unsupported(_)) => {}
					Err(error) => return Err(error),
				}
			}
			let temp = tempfile::Builder::new()
				.prefix("wingdrive-cloud-copy-")
				.tempfile()?
				.into_temp_path();
			let bytes = from_backend.read_to_file(from, &temp).await?;
			to_backend.write_from_file(to, &temp).await?;
			drop(temp);
			Ok(bytes)
		}
	}
}

/// Running totals of a cloud transfer.
#[derive(Debug, Default)]
pub struct TransferTotals {
	pub copied: usize,
	pub skipped: usize,
	pub bytes: u64,
	pub failed: Vec<CopyError>,
}

/// Picks "name (2).ext", "name (3).ext", ... until the name is free.
async fn free_name(destination: &Endpoint) -> Result<Endpoint, VolumeError> {
	let name = destination.name().unwrap_or_default();
	let path = Path::new(&name);
	let stem = path
		.file_stem()
		.map(|s| s.to_string_lossy().into_owned())
		.unwrap_or_default();
	let extension = path
		.extension()
		.map(|e| format!(".{}", e.to_string_lossy()))
		.unwrap_or_default();
	for counter in 2.. {
		let candidate = destination.with_name(&format!("{stem} ({counter}){extension}"));
		if candidate.kind().await?.is_none() {
			return Ok(candidate);
		}
	}
	unreachable!("an unbounded counter always finds a free name")
}

/// Copies or moves `source` to exactly `destination`, recursing into folders.
///
/// Returns `Ok(false)` when the item was skipped because of a conflict.
pub fn transfer_tree<'a>(
	source: &'a Endpoint,
	destination: &'a Endpoint,
	is_move: bool,
	conflict: Option<FileConflictResolution>,
	totals: &'a mut TransferTotals,
	on_file: &'a (dyn Fn(&str, &TransferTotals) + Send + Sync),
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, VolumeError>> + Send + 'a>> {
	Box::pin(async move {
		let kind = source.kind().await?.ok_or_else(|| {
			VolumeError::Io(std::io::Error::new(
				std::io::ErrorKind::NotFound,
				format!("{} does not exist", source.display()),
			))
		})?;
		let mut destination = destination.clone();
		if let Some(existing) = destination.kind().await? {
			let merge_folders = kind == EntryKind::Directory && existing == EntryKind::Directory;
			if !merge_folders {
				match conflict {
					Some(FileConflictResolution::Overwrite) => destination.remove(existing).await?,
					Some(FileConflictResolution::AutoModifyName) => {
						destination = free_name(&destination).await?
					}
					Some(FileConflictResolution::Skip) => {
						totals.skipped += 1;
						return Ok(false);
					}
					Some(FileConflictResolution::Abort) | None => {
						return Err(VolumeError::Io(std::io::Error::new(
							std::io::ErrorKind::AlreadyExists,
							format!("{} already exists", destination.display()),
						)));
					}
				}
			}
		}

		// A move inside one cloud volume is a server-side rename, folders included.
		if is_move && source.same_volume(&destination) && destination.kind().await?.is_none() {
			if let (
				Endpoint::Cloud {
					backend,
					path: from,
					..
				},
				Endpoint::Cloud { path: to, .. },
			) = (source, &destination)
			{
				match backend.rename(from, to).await {
					Ok(()) => {
						totals.copied += 1;
						on_file(&destination.display(), totals);
						return Ok(true);
					}
					Err(VolumeError::Unsupported(_)) => {}
					Err(error) => return Err(error),
				}
			}
		}

		if kind == EntryKind::Directory {
			destination.create_dir().await?;
			let mut complete = true;
			for name in source.children().await? {
				let child_source = source.join(&name);
				let child_destination = destination.join(&name);
				match transfer_tree(
					&child_source,
					&child_destination,
					is_move,
					conflict,
					totals,
					on_file,
				)
				.await
				{
					Ok(true) => {}
					Ok(false) => complete = false,
					Err(error) => {
						complete = false;
						totals.failed.push(CopyError {
							source: PathBuf::from(child_source.display()),
							destination: PathBuf::from(child_destination.display()),
							error: error.to_string(),
						});
					}
				}
			}
			// Keep a source folder that still holds skipped or failed items.
			if is_move && complete {
				source.remove(EntryKind::Directory).await?;
			}
			return Ok(complete);
		}

		if kind == EntryKind::Symlink {
			// Links point at local paths that mean nothing on another storage service.
			totals.skipped += 1;
			return Ok(false);
		}

		let bytes = transfer_file(source, &destination).await?;
		totals.copied += 1;
		totals.bytes += bytes;
		if is_move {
			source.remove(EntryKind::File).await?;
		}
		on_file(&destination.display(), totals);
		Ok(true)
	})
}

/// Runs a whole `FileCopyJob` batch that touches a cloud volume.
pub async fn run_cloud_batch(
	ctx: &JobContext<'_>,
	sources: &[WingPath],
	destination: &WingPath,
	move_mode: Option<&MoveMode>,
	is_move: bool,
	conflict: Option<FileConflictResolution>,
	started_at: Instant,
) -> JobResult<FileCopyOutput> {
	let volumes = ctx.volume_manager();
	let destination_endpoint = Endpoint::resolve(destination, volumes.as_deref())
		.await
		.map_err(JobError::execution)?;
	let destination_is_folder = destination_endpoint
		.kind()
		.await
		.map_err(|e| JobError::execution(e.to_string()))?
		== Some(EntryKind::Directory);
	let rename = matches!(move_mode, Some(MoveMode::Rename));
	if !rename && !destination_is_folder && sources.len() > 1 {
		destination_endpoint
			.create_dir()
			.await
			.map_err(|e| JobError::execution(e.to_string()))?;
	}

	let mut totals = TransferTotals::default();
	let total_sources = sources.len();
	let report = |current: &str, totals: &TransferTotals| {
		let progress = CopyProgress {
			phase: CopyPhase::Copying,
			current_file: current.to_string(),
			current_source_path: None,
			files_copied: totals.copied,
			total_files: total_sources.max(totals.copied),
			bytes_copied: totals.bytes,
			total_bytes: 0,
			current_operation: if is_move { "Moving" } else { "Copying" }.to_string(),
			estimated_remaining: None,
			preparation_complete: true,
			error_count: totals.failed.len(),
			transfer_rate: 0.0,
			elapsed: Some(started_at.elapsed()),
			strategy_metadata: None,
		};
		ctx.progress(Progress::generic(progress.to_generic_progress()));
	};

	for source in sources {
		ctx.check_interrupt().await?;
		let outcome = async {
			let source_endpoint = Endpoint::resolve(source, volumes.as_deref())
				.await
				.map_err(|e| VolumeError::InvalidData(e))?;
			let target = if rename || (!destination_is_folder && total_sources == 1) {
				destination_endpoint.clone()
			} else {
				let name = source_endpoint
					.name()
					.ok_or_else(|| VolumeError::InvalidData("source has no name".into()))?;
				destination_endpoint.join(&name)
			};
			ctx.log(format!(
				"{} {} -> {}",
				if is_move { "Moving" } else { "Copying" },
				source_endpoint.display(),
				target.display()
			));
			transfer_tree(
				&source_endpoint,
				&target,
				is_move,
				conflict,
				&mut totals,
				&report,
			)
			.await
		}
		.await;
		if let Err(error) = outcome {
			ctx.add_non_critical_error(format!("{}: {error}", source.display()));
			totals.failed.push(CopyError {
				source: PathBuf::from(source.display()),
				destination: PathBuf::from(destination.display()),
				error: error.to_string(),
			});
		}
	}

	ctx.log(format!(
		"Cloud {} finished: {} files, {} bytes, {} skipped, {} failed",
		if is_move { "move" } else { "copy" },
		totals.copied,
		totals.bytes,
		totals.skipped,
		totals.failed.len()
	));
	if totals.copied == 0 && !totals.failed.is_empty() && totals.skipped == 0 {
		return Err(JobError::execution(
			totals
				.failed
				.iter()
				.map(|f| format!("{}: {}", f.source.display(), f.error))
				.collect::<Vec<_>>()
				.join("; "),
		));
	}
	Ok(FileCopyOutput {
		copied_count: totals.copied,
		failed_count: totals.failed.len(),
		skipped_count: totals.skipped,
		total_bytes: totals.bytes,
		duration: started_at.elapsed(),
		failed_copies: totals.failed,
		is_move_operation: is_move,
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::volume::backend::LocalBackend;

	fn cloud(root: &Path, volume: &str, path: &str) -> Endpoint {
		Endpoint::Cloud {
			backend: Arc::new(LocalBackend::new(root)),
			path: PathBuf::from(path),
			volume: volume.to_string(),
		}
	}

	fn noop(_: &str, _: &TransferTotals) {}

	#[tokio::test]
	async fn upload_download_and_cross_volume_copy() {
		let local = tempfile::tempdir().unwrap();
		let a = tempfile::tempdir().unwrap();
		let b = tempfile::tempdir().unwrap();
		std::fs::create_dir_all(local.path().join("album/sub")).unwrap();
		std::fs::write(local.path().join("album/one.jpg"), "1").unwrap();
		std::fs::write(local.path().join("album/sub/two.jpg"), "22").unwrap();

		let mut totals = TransferTotals::default();
		transfer_tree(
			&Endpoint::Local(local.path().join("album")),
			&cloud(a.path(), "gdrive://a", "album"),
			false,
			None,
			&mut totals,
			&noop,
		)
		.await
		.unwrap();
		assert_eq!((totals.copied, totals.bytes), (2, 3));
		assert_eq!(
			std::fs::read_to_string(a.path().join("album/sub/two.jpg")).unwrap(),
			"22"
		);

		// Cloud to cloud on different volumes goes through a temp file.
		let mut totals = TransferTotals::default();
		transfer_tree(
			&cloud(a.path(), "gdrive://a", "album/one.jpg"),
			&cloud(b.path(), "s3://b", "one.jpg"),
			false,
			None,
			&mut totals,
			&noop,
		)
		.await
		.unwrap();
		assert_eq!(
			std::fs::read_to_string(b.path().join("one.jpg")).unwrap(),
			"1"
		);

		// Download with a move deletes the cloud source.
		let mut totals = TransferTotals::default();
		transfer_tree(
			&cloud(b.path(), "s3://b", "one.jpg"),
			&Endpoint::Local(local.path().join("back.jpg")),
			true,
			None,
			&mut totals,
			&noop,
		)
		.await
		.unwrap();
		assert_eq!(
			std::fs::read_to_string(local.path().join("back.jpg")).unwrap(),
			"1"
		);
		assert!(!b.path().join("one.jpg").exists());
	}

	#[tokio::test]
	async fn same_volume_move_is_a_rename() {
		let a = tempfile::tempdir().unwrap();
		std::fs::create_dir_all(a.path().join("x/deep")).unwrap();
		std::fs::write(a.path().join("x/deep/f.txt"), "f").unwrap();
		let mut totals = TransferTotals::default();
		transfer_tree(
			&cloud(a.path(), "gdrive://a", "x"),
			&cloud(a.path(), "gdrive://a", "y"),
			true,
			None,
			&mut totals,
			&noop,
		)
		.await
		.unwrap();
		assert_eq!(totals.copied, 1, "one rename for the whole folder");
		assert!(a.path().join("y/deep/f.txt").exists());
		assert!(!a.path().join("x").exists());
	}

	#[tokio::test]
	async fn conflicts_follow_the_resolution() {
		let local = tempfile::tempdir().unwrap();
		let a = tempfile::tempdir().unwrap();
		std::fs::write(local.path().join("f.txt"), "new").unwrap();
		std::fs::write(a.path().join("f.txt"), "old").unwrap();
		let source = Endpoint::Local(local.path().join("f.txt"));
		let target = cloud(a.path(), "gdrive://a", "f.txt");

		let mut totals = TransferTotals::default();
		let error = transfer_tree(&source, &target, false, None, &mut totals, &noop)
			.await
			.unwrap_err();
		assert!(error.to_string().contains("already exists"));

		assert!(!transfer_tree(
			&source,
			&target,
			false,
			Some(FileConflictResolution::Skip),
			&mut totals,
			&noop
		)
		.await
		.unwrap());
		assert_eq!(
			std::fs::read_to_string(a.path().join("f.txt")).unwrap(),
			"old"
		);

		transfer_tree(
			&source,
			&target,
			false,
			Some(FileConflictResolution::AutoModifyName),
			&mut totals,
			&noop,
		)
		.await
		.unwrap();
		assert_eq!(
			std::fs::read_to_string(a.path().join("f (2).txt")).unwrap(),
			"new"
		);

		transfer_tree(
			&source,
			&target,
			false,
			Some(FileConflictResolution::Overwrite),
			&mut totals,
			&noop,
		)
		.await
		.unwrap();
		assert_eq!(
			std::fs::read_to_string(a.path().join("f.txt")).unwrap(),
			"new"
		);
	}
}
