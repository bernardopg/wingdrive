//! # Cloud folder sync engine
//!
//! Keeps a local folder and a folder on a cloud volume in step. Each run takes
//! a snapshot of both trees, compares them with the manifest saved by the last
//! successful run, and plans the smallest set of transfers. The manifest is
//! what makes deletions safe: a file missing on one side counts as deleted
//! only if the manifest says both sides had it before; otherwise it is new on
//! the other side.
//!
//! Three modes exist. `TwoWay` sends each side's changes to the other and, when
//! both sides changed the same file, keeps both by renaming the local copy to
//! "name (conflict copy ...).ext". `Upload` and `Download` mirror one side onto
//! the other: the source overwrites changes on the target. In every mode a
//! deletion only propagates when `propagate_deletes` is on, and only for files
//! that were synced before, so a first run never deletes anything. Local
//! deletions go to the system trash and remote ones to the service's trash
//! when it has one.
//!
//! Planning (`plan`) is a pure function over two snapshots and the manifest,
//! which is where the rules above are tested. `run` does the I/O.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use specta::Type;
use tracing::{debug, warn};

use crate::volume::backend::VolumeBackend;
use crate::volume::error::VolumeError;

/// Which way changes flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
	/// Changes on either side reach the other.
	TwoWay,
	/// The local folder is the source; the cloud folder mirrors it.
	Upload,
	/// The cloud folder is the source; the local folder mirrors it.
	Download,
}

impl SyncMode {
	pub fn as_str(self) -> &'static str {
		match self {
			Self::TwoWay => "two_way",
			Self::Upload => "upload",
			Self::Download => "download",
		}
	}

	pub fn parse(value: &str) -> Option<Self> {
		match value {
			"two_way" => Some(Self::TwoWay),
			"upload" => Some(Self::Upload),
			"download" => Some(Self::Download),
			_ => None,
		}
	}
}

/// What WingDrive knows about one side of a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FileState {
	pub size: u64,
	/// Modification time in whole seconds; finer precision varies across filesystems.
	pub modified: Option<i64>,
	/// Service revision such as an MD5 or ETag, remote side only.
	pub revision: Option<String>,
}

impl FileState {
	/// Whether `self` differs from the state recorded at the last sync.
	fn changed_since(&self, recorded: &FileState) -> bool {
		match (&self.revision, &recorded.revision) {
			(Some(now), Some(then)) => now != then,
			_ => self.size != recorded.size || self.modified != recorded.modified,
		}
	}
}

/// Both sides of a file as they were right after the last successful sync.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestEntry {
	pub local: FileState,
	pub remote: FileState,
}

/// What the last run left in sync, keyed by relative path with "/" separators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Manifest {
	#[serde(default)]
	pub files: BTreeMap<String, ManifestEntry>,
	#[serde(default)]
	pub dirs: BTreeSet<String>,
}

/// One side of the pair at the time of a run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeSnapshot {
	pub files: BTreeMap<String, FileState>,
	pub dirs: BTreeSet<String>,
}

/// A step of a sync run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncAction {
	Upload(String),
	Download(String),
	/// Both sides changed: move the local file aside, then download and upload.
	KeepBoth(String),
	DeleteLocal(String),
	DeleteRemote(String),
	CreateLocalDir(String),
	CreateRemoteDir(String),
	DeleteLocalDir(String),
	DeleteRemoteDir(String),
	/// Record the file as synced without transferring, for identical first-run files.
	Adopt(String),
	/// Drop a manifest entry for a file gone from both sides.
	Forget(String),
}

/// Files that never sync: partial downloads, OS metadata and editor lock files.
pub fn is_ignored(name: &str) -> bool {
	name.ends_with(".wingpart")
		|| name == ".DS_Store"
		|| name == "Thumbs.db"
		|| name == "desktop.ini"
		|| name.starts_with("~$")
		|| (name.starts_with(".~lock.") && name.ends_with('#'))
}

/// Plans a run. Pure, so every rule in the module docs is unit tested.
pub fn plan(
	mode: SyncMode,
	propagate_deletes: bool,
	local: &TreeSnapshot,
	remote: &TreeSnapshot,
	manifest: &Manifest,
) -> Vec<SyncAction> {
	use SyncAction::*;

	let upload_allowed = mode != SyncMode::Download;
	let download_allowed = mode != SyncMode::Upload;
	let mut actions = Vec::new();

	// Folders first, so file transfers have a parent to land in.
	let all_dirs: BTreeSet<&String> = local
		.dirs
		.iter()
		.chain(remote.dirs.iter())
		.chain(manifest.dirs.iter())
		.collect();
	let mut dir_deletes = Vec::new();
	for dir in all_dirs {
		let (l, r, known) = (
			local.dirs.contains(dir),
			remote.dirs.contains(dir),
			manifest.dirs.contains(dir),
		);
		match (l, r, known) {
			(true, false, false) if upload_allowed => actions.push(CreateRemoteDir(dir.clone())),
			(false, true, false) if download_allowed => actions.push(CreateLocalDir(dir.clone())),
			(true, false, true) => match mode {
				SyncMode::Upload => actions.push(CreateRemoteDir(dir.clone())),
				_ if propagate_deletes => dir_deletes.push(DeleteLocalDir(dir.clone())),
				SyncMode::TwoWay => actions.push(CreateRemoteDir(dir.clone())),
				_ => {}
			},
			(false, true, true) => match mode {
				SyncMode::Download => actions.push(CreateLocalDir(dir.clone())),
				_ if propagate_deletes => dir_deletes.push(DeleteRemoteDir(dir.clone())),
				SyncMode::TwoWay => actions.push(CreateLocalDir(dir.clone())),
				_ => {}
			},
			_ => {}
		}
	}

	let all_files: BTreeSet<&String> = local
		.files
		.keys()
		.chain(remote.files.keys())
		.chain(manifest.files.keys())
		.collect();
	for path in all_files {
		let l = local.files.get(path);
		let r = remote.files.get(path);
		let m = manifest.files.get(path);
		let action = match (l, r, m) {
			(Some(l), Some(r), Some(m)) => {
				let lc = l.changed_since(&m.local);
				let rc = r.changed_since(&m.remote);
				match mode {
					SyncMode::TwoWay => match (lc, rc) {
						(false, false) => None,
						(true, false) => Some(Upload(path.clone())),
						(false, true) => Some(Download(path.clone())),
						(true, true) => Some(KeepBoth(path.clone())),
					},
					SyncMode::Upload => (lc || rc).then(|| Upload(path.clone())),
					SyncMode::Download => (lc || rc).then(|| Download(path.clone())),
				}
			}
			(Some(l), None, Some(m)) => match mode {
				SyncMode::Upload => Some(Upload(path.clone())),
				SyncMode::TwoWay if l.changed_since(&m.local) || !propagate_deletes => {
					Some(Upload(path.clone()))
				}
				_ if propagate_deletes => Some(DeleteLocal(path.clone())),
				_ => Some(Forget(path.clone())),
			},
			(None, Some(r), Some(m)) => match mode {
				SyncMode::Download => Some(Download(path.clone())),
				SyncMode::TwoWay if r.changed_since(&m.remote) || !propagate_deletes => {
					Some(Download(path.clone()))
				}
				_ if propagate_deletes => Some(DeleteRemote(path.clone())),
				_ => Some(Forget(path.clone())),
			},
			(Some(_), None, None) => upload_allowed.then(|| Upload(path.clone())),
			(None, Some(_), None) => download_allowed.then(|| Download(path.clone())),
			(Some(l), Some(r), None) => {
				let same_size = l.size == r.size;
				match mode {
					SyncMode::TwoWay if same_size => Some(Adopt(path.clone())),
					SyncMode::TwoWay => Some(KeepBoth(path.clone())),
					SyncMode::Upload if same_size => Some(Adopt(path.clone())),
					SyncMode::Upload => Some(Upload(path.clone())),
					SyncMode::Download if same_size => Some(Adopt(path.clone())),
					SyncMode::Download => Some(Download(path.clone())),
				}
			}
			(None, None, Some(_)) => Some(Forget(path.clone())),
			(None, None, None) => None,
		};
		actions.extend(action);
	}

	// Deepest folders first, after their files are gone.
	dir_deletes.sort_by_key(|action| match action {
		DeleteLocalDir(p) | DeleteRemoteDir(p) => std::cmp::Reverse(p.matches('/').count()),
		_ => std::cmp::Reverse(0),
	});
	actions.extend(dir_deletes);
	actions
}

fn seconds(time: Option<SystemTime>) -> Option<i64> {
	time.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
		.map(|d| d.as_secs() as i64)
}

fn join_relative(root: &Path, relative: &str) -> PathBuf {
	relative
		.split('/')
		.filter(|part| !part.is_empty())
		.fold(root.to_path_buf(), |path, part| path.join(part))
}

/// Walks the local folder. Symlinks are skipped so a link cannot pull files
/// from outside the folder into the cloud.
pub async fn scan_local(root: &Path) -> std::io::Result<TreeSnapshot> {
	let root = root.to_path_buf();
	tokio::task::spawn_blocking(move || {
		let mut snapshot = TreeSnapshot::default();
		let mut stack = vec![(root.clone(), String::new())];
		while let Some((dir, prefix)) = stack.pop() {
			for entry in std::fs::read_dir(&dir)? {
				let entry = entry?;
				let name = entry.file_name().to_string_lossy().into_owned();
				if is_ignored(&name) {
					continue;
				}
				let file_type = entry.file_type()?;
				let relative = if prefix.is_empty() {
					name.clone()
				} else {
					format!("{prefix}/{name}")
				};
				if file_type.is_dir() {
					snapshot.dirs.insert(relative.clone());
					stack.push((entry.path(), relative));
				} else if file_type.is_file() {
					let metadata = entry.metadata()?;
					snapshot.files.insert(
						relative,
						FileState {
							size: metadata.len(),
							modified: seconds(metadata.modified().ok()),
							revision: None,
						},
					);
				}
			}
		}
		Ok(snapshot)
	})
	.await
	.map_err(|e| std::io::Error::other(e.to_string()))?
}

/// Walks the cloud folder through its backend.
pub async fn scan_remote(
	backend: &dyn VolumeBackend,
	root: &Path,
) -> Result<TreeSnapshot, VolumeError> {
	let mut snapshot = TreeSnapshot::default();
	let mut stack = vec![(root.to_path_buf(), String::new())];
	while let Some((dir, prefix)) = stack.pop() {
		for entry in backend.read_dir(&dir).await? {
			if is_ignored(&entry.name) {
				continue;
			}
			let relative = if prefix.is_empty() {
				entry.name.clone()
			} else {
				format!("{prefix}/{}", entry.name)
			};
			match entry.kind {
				crate::ops::indexing::state::EntryKind::Directory => {
					snapshot.dirs.insert(relative.clone());
					stack.push((dir.join(&entry.name), relative));
				}
				crate::ops::indexing::state::EntryKind::File => {
					snapshot.files.insert(
						relative,
						FileState {
							size: entry.size,
							modified: seconds(entry.modified),
							revision: entry.revision,
						},
					);
				}
				_ => {}
			}
		}
	}
	Ok(snapshot)
}

/// Counts of what a run did.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SyncReport {
	pub uploaded: u32,
	pub downloaded: u32,
	pub deleted_local: u32,
	pub deleted_remote: u32,
	pub conflicts: u32,
	pub bytes_transferred: u64,
	/// Paths that failed with their error; the next run retries them.
	pub errors: Vec<String>,
}

/// Progress callback: (actions done, total actions, current path).
pub type ProgressFn<'a> = &'a (dyn Fn(usize, usize, &str) + Send + Sync);

/// Where the run happens.
pub struct SyncTarget {
	pub local_root: PathBuf,
	pub backend: Arc<dyn VolumeBackend>,
	pub remote_root: PathBuf,
	/// Send local deletions to the system trash instead of deleting them.
	pub use_system_trash: bool,
}

async fn local_state(path: &Path) -> std::io::Result<FileState> {
	let metadata = tokio::fs::metadata(path).await?;
	Ok(FileState {
		size: metadata.len(),
		modified: seconds(metadata.modified().ok()),
		revision: None,
	})
}

async fn remote_state(backend: &dyn VolumeBackend, path: &Path) -> Result<FileState, VolumeError> {
	let metadata = backend.metadata(path).await?;
	Ok(FileState {
		size: metadata.size,
		modified: seconds(metadata.modified),
		revision: metadata.revision,
	})
}

async fn trash_local(path: &Path, use_system_trash: bool) -> std::io::Result<()> {
	let _ = use_system_trash;
	#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
	if use_system_trash {
		let owned = path.to_path_buf();
		let trashed = tokio::task::spawn_blocking(move || trash::delete(&owned))
			.await
			.map_err(|e| std::io::Error::other(e.to_string()))?;
		if trashed.is_ok() {
			return Ok(());
		}
		warn!(path = %path.display(), "System trash unavailable, deleting synced file permanently");
	}
	tokio::fs::remove_file(path).await
}

async fn trash_remote(backend: &dyn VolumeBackend, path: &Path) -> Result<(), VolumeError> {
	match backend.trash(path).await {
		Err(VolumeError::Unsupported(_)) => backend.delete(path).await,
		other => other,
	}
}

async fn upload(
	target: &SyncTarget,
	relative: &str,
) -> Result<(FileState, FileState, u64), VolumeError> {
	let local = join_relative(&target.local_root, relative);
	let remote = join_relative(&target.remote_root, relative);
	let bytes = target.backend.write_from_file(&remote, &local).await?;
	Ok((
		local_state(&local).await?,
		remote_state(target.backend.as_ref(), &remote).await?,
		bytes,
	))
}

async fn download(
	target: &SyncTarget,
	relative: &str,
) -> Result<(FileState, FileState, u64), VolumeError> {
	let local = join_relative(&target.local_root, relative);
	let remote = join_relative(&target.remote_root, relative);
	if let Some(parent) = local.parent() {
		tokio::fs::create_dir_all(parent).await?;
	}
	let bytes = target.backend.read_to_file(&remote, &local).await?;
	Ok((
		local_state(&local).await?,
		remote_state(target.backend.as_ref(), &remote).await?,
		bytes,
	))
}

/// Runs one sync and returns the report and the new manifest.
///
/// A failing path is recorded in the report and left out of the manifest
/// update, so the next run sees it as still pending. The run stops early only
/// when a snapshot cannot be taken.
pub async fn run(
	target: &SyncTarget,
	mode: SyncMode,
	propagate_deletes: bool,
	manifest: &Manifest,
	progress: ProgressFn<'_>,
) -> Result<(SyncReport, Manifest), VolumeError> {
	tokio::fs::create_dir_all(&target.local_root).await?;
	target
		.backend
		.create_directory(&target.remote_root, true)
		.await?;
	let local = scan_local(&target.local_root).await?;
	let remote = scan_remote(target.backend.as_ref(), &target.remote_root).await?;
	let actions = plan(mode, propagate_deletes, &local, &remote, manifest);
	debug!(actions = actions.len(), "Planned cloud sync");

	let mut report = SyncReport::default();
	let mut next = manifest.clone();
	let total = actions.len();
	for (index, action) in actions.iter().enumerate() {
		let path = match action {
			SyncAction::Upload(p)
			| SyncAction::Download(p)
			| SyncAction::KeepBoth(p)
			| SyncAction::DeleteLocal(p)
			| SyncAction::DeleteRemote(p)
			| SyncAction::CreateLocalDir(p)
			| SyncAction::CreateRemoteDir(p)
			| SyncAction::DeleteLocalDir(p)
			| SyncAction::DeleteRemoteDir(p)
			| SyncAction::Adopt(p)
			| SyncAction::Forget(p) => p.clone(),
		};
		progress(index, total, &path);
		let result: Result<(), VolumeError> = async {
			match action {
				SyncAction::Upload(p) => {
					let (l, r, bytes) = upload(target, p).await?;
					next.files.insert(
						p.clone(),
						ManifestEntry {
							local: l,
							remote: r,
						},
					);
					report.uploaded += 1;
					report.bytes_transferred += bytes;
				}
				SyncAction::Download(p) => {
					let (l, r, bytes) = download(target, p).await?;
					next.files.insert(
						p.clone(),
						ManifestEntry {
							local: l,
							remote: r,
						},
					);
					report.downloaded += 1;
					report.bytes_transferred += bytes;
				}
				SyncAction::KeepBoth(p) => {
					let local = join_relative(&target.local_root, p);
					let copy = crate::service::integrations::cloud_edit::conflict_copy_path(&local);
					tokio::fs::rename(&local, &copy).await?;
					let copy_relative = copy
						.strip_prefix(&target.local_root)
						.map(|c| c.to_string_lossy().replace('\\', "/"))
						.map_err(|e| VolumeError::InvalidData(e.to_string()))?;
					let (l, r, down) = download(target, p).await?;
					next.files.insert(
						p.clone(),
						ManifestEntry {
							local: l,
							remote: r,
						},
					);
					if mode != SyncMode::Download {
						let (l, r, up) = upload(target, &copy_relative).await?;
						next.files.insert(
							copy_relative,
							ManifestEntry {
								local: l,
								remote: r,
							},
						);
						report.bytes_transferred += up;
					}
					report.conflicts += 1;
					report.bytes_transferred += down;
				}
				SyncAction::DeleteLocal(p) => {
					trash_local(
						&join_relative(&target.local_root, p),
						target.use_system_trash,
					)
					.await?;
					next.files.remove(p);
					report.deleted_local += 1;
				}
				SyncAction::DeleteRemote(p) => {
					trash_remote(
						target.backend.as_ref(),
						&join_relative(&target.remote_root, p),
					)
					.await?;
					next.files.remove(p);
					report.deleted_remote += 1;
				}
				SyncAction::CreateLocalDir(p) => {
					tokio::fs::create_dir_all(join_relative(&target.local_root, p)).await?;
					next.dirs.insert(p.clone());
				}
				SyncAction::CreateRemoteDir(p) => {
					target
						.backend
						.create_directory(&join_relative(&target.remote_root, p), true)
						.await?;
					next.dirs.insert(p.clone());
				}
				SyncAction::DeleteLocalDir(p) => {
					// remove_dir refuses non-empty folders, which protects new local files.
					match tokio::fs::remove_dir(join_relative(&target.local_root, p)).await {
						Ok(()) => {
							next.dirs.remove(p);
						}
						Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
							next.dirs.remove(p);
						}
						Err(error) => debug!(%error, path = %p, "Kept non-empty local folder"),
					}
				}
				SyncAction::DeleteRemoteDir(p) => {
					let remote = join_relative(&target.remote_root, p);
					if target.backend.read_dir(&remote).await?.is_empty() {
						trash_remote(target.backend.as_ref(), &remote).await?;
					}
					next.dirs.remove(p);
				}
				SyncAction::Adopt(p) => {
					let l = local_state(&join_relative(&target.local_root, p)).await?;
					let r = remote_state(
						target.backend.as_ref(),
						&join_relative(&target.remote_root, p),
					)
					.await?;
					next.files.insert(
						p.clone(),
						ManifestEntry {
							local: l,
							remote: r,
						},
					);
				}
				SyncAction::Forget(p) => {
					next.files.remove(p);
				}
			}
			Ok(())
		}
		.await;
		if let Err(error) = result {
			warn!(%error, path = %path, "Cloud sync step failed");
			report.errors.push(format!("{path}: {error}"));
		}
	}
	// Folders present on both sides are in sync too.
	for dir in local.dirs.intersection(&remote.dirs) {
		next.dirs.insert(dir.clone());
	}
	progress(total, total, "");
	Ok((report, next))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::volume::backend::LocalBackend;
	use SyncAction::*;

	fn state(size: u64, modified: i64) -> FileState {
		FileState {
			size,
			modified: Some(modified),
			revision: None,
		}
	}

	fn snap(files: &[(&str, FileState)], dirs: &[&str]) -> TreeSnapshot {
		TreeSnapshot {
			files: files
				.iter()
				.map(|(p, s)| (p.to_string(), s.clone()))
				.collect(),
			dirs: dirs.iter().map(|d| d.to_string()).collect(),
		}
	}

	fn manifest(files: &[(&str, FileState, FileState)], dirs: &[&str]) -> Manifest {
		Manifest {
			files: files
				.iter()
				.map(|(p, l, r)| {
					(
						p.to_string(),
						ManifestEntry {
							local: l.clone(),
							remote: r.clone(),
						},
					)
				})
				.collect(),
			dirs: dirs.iter().map(|d| d.to_string()).collect(),
		}
	}

	#[test]
	fn first_run_copies_new_files_both_ways_and_never_deletes() {
		let local = snap(
			&[("a.txt", state(1, 10)), ("same.txt", state(5, 1))],
			&["docs"],
		);
		let remote = snap(
			&[("b.txt", state(2, 20)), ("same.txt", state(5, 99))],
			&["pics"],
		);
		let actions = plan(
			SyncMode::TwoWay,
			true,
			&local,
			&remote,
			&Manifest::default(),
		);
		assert!(actions.contains(&Upload("a.txt".into())));
		assert!(actions.contains(&Download("b.txt".into())));
		assert!(actions.contains(&Adopt("same.txt".into())));
		assert!(actions.contains(&CreateRemoteDir("docs".into())));
		assert!(actions.contains(&CreateLocalDir("pics".into())));
		assert!(!actions
			.iter()
			.any(|a| matches!(a, DeleteLocal(_) | DeleteRemote(_))));
	}

	#[test]
	fn two_way_sends_changes_and_keeps_both_on_conflict() {
		let m = manifest(
			&[
				("l.txt", state(1, 1), state(1, 1)),
				("r.txt", state(1, 1), state(1, 1)),
				("both.txt", state(1, 1), state(1, 1)),
				("same.txt", state(1, 1), state(1, 1)),
			],
			&[],
		);
		let local = snap(
			&[
				("l.txt", state(2, 2)),
				("r.txt", state(1, 1)),
				("both.txt", state(3, 3)),
				("same.txt", state(1, 1)),
			],
			&[],
		);
		let remote = snap(
			&[
				("l.txt", state(1, 1)),
				("r.txt", state(4, 4)),
				("both.txt", state(5, 5)),
				("same.txt", state(1, 1)),
			],
			&[],
		);
		let actions = plan(SyncMode::TwoWay, false, &local, &remote, &m);
		assert_eq!(
			actions,
			vec![
				KeepBoth("both.txt".into()),
				Upload("l.txt".into()),
				Download("r.txt".into())
			]
		);
	}

	#[test]
	fn remote_revision_beats_size_and_time() {
		let recorded = FileState {
			size: 1,
			modified: Some(1),
			revision: Some("md5-a".into()),
		};
		let mut now = recorded.clone();
		now.modified = Some(99);
		assert!(!now.changed_since(&recorded), "same md5, touched timestamp");
		now.revision = Some("md5-b".into());
		assert!(now.changed_since(&recorded));
	}

	#[test]
	fn deletions_propagate_only_when_enabled() {
		let m = manifest(
			&[
				("gone-local.txt", state(1, 1), state(1, 1)),
				("gone-remote.txt", state(1, 1), state(1, 1)),
			],
			&[],
		);
		let local = snap(&[("gone-remote.txt", state(1, 1))], &[]);
		let remote = snap(&[("gone-local.txt", state(1, 1))], &[]);
		let on = plan(SyncMode::TwoWay, true, &local, &remote, &m);
		assert!(on.contains(&DeleteRemote("gone-local.txt".into())));
		assert!(on.contains(&DeleteLocal("gone-remote.txt".into())));
		let off = plan(SyncMode::TwoWay, false, &local, &remote, &m);
		assert!(off.contains(&Download("gone-local.txt".into())));
		assert!(off.contains(&Upload("gone-remote.txt".into())));
	}

	#[test]
	fn edit_wins_over_delete_in_two_way() {
		let m = manifest(&[("doc.txt", state(1, 1), state(1, 1))], &[]);
		let local = snap(&[("doc.txt", state(9, 9))], &[]);
		let actions = plan(SyncMode::TwoWay, true, &local, &TreeSnapshot::default(), &m);
		assert_eq!(actions, vec![Upload("doc.txt".into())]);
	}

	#[test]
	fn mirror_modes_only_touch_the_target() {
		let m = manifest(&[("x.txt", state(1, 1), state(1, 1))], &[]);
		let local = snap(
			&[("x.txt", state(1, 1)), ("new-local.txt", state(1, 1))],
			&[],
		);
		let remote = snap(
			&[("x.txt", state(7, 7)), ("new-remote.txt", state(1, 1))],
			&[],
		);

		let up = plan(SyncMode::Upload, true, &local, &remote, &m);
		assert!(
			up.contains(&Upload("x.txt".into())),
			"local overwrites remote edits"
		);
		assert!(up.contains(&Upload("new-local.txt".into())));
		assert!(!up.iter().any(|a| matches!(a, Download(_) | DeleteLocal(_))));
		assert!(
			!up.contains(&DeleteRemote("new-remote.txt".into())),
			"never synced, never deleted"
		);

		let down = plan(SyncMode::Download, true, &local, &remote, &m);
		assert!(down.contains(&Download("x.txt".into())));
		assert!(down.contains(&Download("new-remote.txt".into())));
		assert!(!down
			.iter()
			.any(|a| matches!(a, Upload(_) | DeleteRemote(_) | DeleteLocal(_))));
	}

	#[test]
	fn folder_deletes_run_deepest_first_after_files() {
		let m = manifest(&[("a/b/f.txt", state(1, 1), state(1, 1))], &["a", "a/b"]);
		let remote = snap(&[("a/b/f.txt", state(1, 1))], &["a", "a/b"]);
		let actions = plan(
			SyncMode::TwoWay,
			true,
			&TreeSnapshot::default(),
			&remote,
			&m,
		);
		assert_eq!(
			actions,
			vec![
				DeleteRemote("a/b/f.txt".into()),
				DeleteRemoteDir("a/b".into()),
				DeleteRemoteDir("a".into()),
			]
		);
	}

	#[test]
	fn ignored_names() {
		for name in [
			"x.wingpart",
			".DS_Store",
			"~$report.docx",
			".~lock.file.odt#",
			"Thumbs.db",
		] {
			assert!(is_ignored(name), "{name}");
		}
		assert!(!is_ignored("report.docx"));
	}

	fn write(root: &Path, relative: &str, content: &str) {
		let path = join_relative(root, relative);
		std::fs::create_dir_all(path.parent().unwrap()).unwrap();
		std::fs::write(path, content).unwrap();
	}

	fn read(root: &Path, relative: &str) -> Option<String> {
		std::fs::read_to_string(join_relative(root, relative)).ok()
	}

	fn bump_mtime(root: &Path, relative: &str, seconds_ahead: u64) {
		let path = join_relative(root, relative);
		let file = std::fs::File::options().write(true).open(path).unwrap();
		file.set_modified(SystemTime::now() + std::time::Duration::from_secs(seconds_ahead))
			.unwrap();
	}

	#[tokio::test]
	async fn end_to_end_two_way_against_a_backend() {
		let local = tempfile::tempdir().unwrap();
		let remote = tempfile::tempdir().unwrap();
		let target = SyncTarget {
			local_root: local.path().to_path_buf(),
			backend: Arc::new(LocalBackend::new(remote.path())),
			remote_root: PathBuf::from("Backup"),
			use_system_trash: false,
		};
		let backup = remote.path().join("Backup");
		write(local.path(), "notes/a.txt", "local a");
		write(local.path(), ".DS_Store", "junk");
		std::fs::create_dir_all(&backup).unwrap();
		write(&backup, "photos/b.jpg", "remote b");
		let noop = |_: usize, _: usize, _: &str| {};

		let (report, manifest) = run(&target, SyncMode::TwoWay, true, &Manifest::default(), &noop)
			.await
			.unwrap();
		assert!(report.errors.is_empty(), "{:?}", report.errors);
		assert_eq!((report.uploaded, report.downloaded), (1, 1));
		assert_eq!(read(&backup, "notes/a.txt").as_deref(), Some("local a"));
		assert_eq!(
			read(local.path(), "photos/b.jpg").as_deref(),
			Some("remote b")
		);
		assert!(read(&backup, ".DS_Store").is_none());

		// Nothing changed: the second run does nothing.
		let (report, manifest) = run(&target, SyncMode::TwoWay, true, &manifest, &noop)
			.await
			.unwrap();
		assert_eq!(report, SyncReport::default());

		// Edit on each side, then delete one file remotely.
		write(local.path(), "notes/a.txt", "local a v2");
		bump_mtime(local.path(), "notes/a.txt", 5);
		write(&backup, "photos/b.jpg", "remote b v2!");
		let (report, manifest) = run(&target, SyncMode::TwoWay, true, &manifest, &noop)
			.await
			.unwrap();
		assert_eq!((report.uploaded, report.downloaded), (1, 1));
		assert_eq!(read(&backup, "notes/a.txt").as_deref(), Some("local a v2"));
		assert_eq!(
			read(local.path(), "photos/b.jpg").as_deref(),
			Some("remote b v2!")
		);

		std::fs::remove_file(backup.join("photos/b.jpg")).unwrap();
		let (report, manifest) = run(&target, SyncMode::TwoWay, true, &manifest, &noop)
			.await
			.unwrap();
		assert_eq!(report.deleted_local, 1);
		assert!(read(local.path(), "photos/b.jpg").is_none());

		// Conflict: both sides edit the same file.
		write(local.path(), "notes/a.txt", "mine!");
		bump_mtime(local.path(), "notes/a.txt", 10);
		write(&backup, "notes/a.txt", "theirs!!");
		let (report, _) = run(&target, SyncMode::TwoWay, true, &manifest, &noop)
			.await
			.unwrap();
		assert_eq!(report.conflicts, 1, "{:?}", report.errors);
		assert_eq!(
			read(local.path(), "notes/a.txt").as_deref(),
			Some("theirs!!")
		);
		let copies: Vec<String> = std::fs::read_dir(backup.join("notes"))
			.unwrap()
			.map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
			.filter(|n| n.contains("conflict copy"))
			.collect();
		assert_eq!(copies.len(), 1);
		assert_eq!(
			read(&backup.join("notes"), &copies[0]).as_deref(),
			Some("mine!")
		);
	}
}
