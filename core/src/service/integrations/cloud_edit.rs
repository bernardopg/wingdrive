//! # Editing cloud files with local apps
//!
//! Desktop apps open local files, so opening a file that lives on a cloud
//! volume downloads it into a private folder under the data directory, hands
//! that path to the app, and watches the copy. Each time the app saves and the
//! file stays unchanged for `SETTLE_TIME`, the copy is uploaded back.
//!
//! The watcher polls metadata instead of using inotify because editors often
//! save by writing a temporary file and renaming it over the original, which
//! breaks a watch on the original inode. Polling a single file every
//! `POLL_INTERVAL` costs one `stat` call.
//!
//! Before each upload the session compares the remote revision with the one it
//! downloaded. If someone changed the file in the cloud meanwhile, the local
//! edit is uploaded next to it as "name (conflict copy).ext" so neither edit is lost.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::domain::addressing::WingPath;
use crate::volume::backend::VolumeBackend;
use crate::volume::error::VolumeError;

const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// How long a changed file must stay unchanged before it is uploaded, so a
/// save in progress is not uploaded half written.
const SETTLE_TIME: Duration = Duration::from_secs(2);

/// Session folders older than this are removed when the daemon starts.
const STALE_SESSION_AGE: Duration = Duration::from_secs(7 * 24 * 3600);

/// Largest file Quick Preview downloads; bigger files open with an app instead.
pub const PREVIEW_MAX_BYTES: u64 = 512 * 1024 * 1024;

/// Folder name of the preview cache, next to the editing sessions.
const PREVIEW_DIR: &str = "preview";

/// Upload state of an editing session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum CloudEditState {
	/// The local copy matches the cloud.
	Synced,
	/// The app saved changes that are waiting to settle.
	Modified,
	Uploading,
	/// The cloud file changed too; the local edit went to `copy_name`.
	Conflict {
		copy_name: String,
	},
	Error {
		message: String,
	},
}

/// A cloud file open for editing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct CloudEditSessionInfo {
	pub id: Uuid,
	pub path: WingPath,
	pub local_path: String,
	pub state: CloudEditState,
	pub last_synced_at: Option<DateTime<Utc>>,
}

/// What opening a cloud file produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CloudEditOpen {
	/// Open this local copy with the default app.
	Local { session: CloudEditSessionInfo },
	/// The file only edits in the browser; open this URL.
	Browser { url: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Snapshot {
	size: u64,
	modified: Option<SystemTime>,
}

async fn snapshot(path: &Path) -> Option<Snapshot> {
	let metadata = tokio::fs::metadata(path).await.ok()?;
	Some(Snapshot {
		size: metadata.len(),
		modified: metadata.modified().ok(),
	})
}

struct SessionState {
	info: CloudEditSessionInfo,
	baseline: Option<Snapshot>,
	remote_revision: Option<String>,
}

struct EditSession {
	backend: Arc<dyn VolumeBackend>,
	remote_path: PathBuf,
	state: Arc<Mutex<SessionState>>,
	task: tokio::task::JoinHandle<()>,
}

/// Open editing sessions of this daemon.
pub struct CloudEditService {
	root: PathBuf,
	sessions: Mutex<HashMap<Uuid, EditSession>>,
}

impl CloudEditService {
	pub fn new(root: PathBuf) -> Self {
		Self {
			root,
			sessions: Mutex::new(HashMap::new()),
		}
	}

	/// Folder that holds the local copies.
	pub fn root(&self) -> &Path {
		&self.root
	}

	/// Downloads a read-only copy of a cloud file for Quick Preview.
	///
	/// The copy is cached by path and revision, so previewing the same file
	/// twice downloads it once, and a file changed in the cloud is fetched
	/// again. Files over `PREVIEW_MAX_BYTES` are refused so a preview never
	/// starts a multi-gigabyte download.
	pub async fn preview_copy(
		&self,
		path: &WingPath,
		backend: &dyn VolumeBackend,
		remote_path: &Path,
	) -> Result<PathBuf, VolumeError> {
		use sha2::{Digest, Sha256};

		let metadata = backend.metadata(remote_path).await?;
		if metadata.kind == crate::ops::indexing::state::EntryKind::Directory {
			return Err(VolumeError::Unsupported("folders have no preview".into()));
		}
		if metadata.size > PREVIEW_MAX_BYTES {
			return Err(VolumeError::Unsupported(format!(
				"files over {} MB are not previewed; open the file instead",
				PREVIEW_MAX_BYTES / (1024 * 1024)
			)));
		}
		let file_name = remote_path
			.file_name()
			.map(|n| n.to_os_string())
			.ok_or_else(|| VolumeError::InvalidData("cloud path has no file name".into()))?;
		let key = Sha256::digest(
			format!(
				"{}\n{}\n{}",
				path.display(),
				metadata.revision.as_deref().unwrap_or_default(),
				metadata.size
			)
			.as_bytes(),
		);
		let folder = self.root.join(PREVIEW_DIR).join(hex::encode(&key[..16]));
		let local = folder.join(file_name);
		if tokio::fs::metadata(&local).await.is_ok() {
			return Ok(local);
		}
		tokio::fs::create_dir_all(&folder).await?;
		backend.read_to_file(remote_path, &local).await?;
		Ok(local)
	}

	/// Deletes session folders left from earlier runs that are older than a week.
	pub async fn remove_stale_sessions(&self) {
		for dir in [self.root.clone(), self.root.join(PREVIEW_DIR)] {
			Self::remove_old_entries(&dir).await;
		}
	}

	async fn remove_old_entries(dir: &Path) {
		let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
			return;
		};
		while let Ok(Some(entry)) = entries.next_entry().await {
			if entry.file_name() == PREVIEW_DIR {
				continue;
			}
			let old = entry
				.metadata()
				.await
				.ok()
				.and_then(|m| m.modified().ok())
				.and_then(|m| m.elapsed().ok())
				.is_some_and(|age| age > STALE_SESSION_AGE);
			if old {
				let _ = tokio::fs::remove_dir_all(entry.path()).await;
			}
		}
	}

	/// Opens `path` for editing.
	///
	/// Opening a file that already has a session returns that session, so a
	/// second double-click does not overwrite unsaved local changes.
	pub async fn open(
		&self,
		path: WingPath,
		backend: Arc<dyn VolumeBackend>,
		remote_path: PathBuf,
	) -> Result<CloudEditOpen, VolumeError> {
		if backend.edits_in_browser(&remote_path).await? {
			let url = backend.web_link(&remote_path).await?.ok_or_else(|| {
				VolumeError::Unsupported("this file has no web editor link".to_string())
			})?;
			return Ok(CloudEditOpen::Browser { url });
		}

		{
			let sessions = self.sessions.lock().await;
			for session in sessions.values() {
				let state = session.state.lock().await;
				if state.info.path == path {
					return Ok(CloudEditOpen::Local {
						session: state.info.clone(),
					});
				}
			}
		}

		let metadata = backend.metadata(&remote_path).await?;
		if metadata.kind == crate::ops::indexing::state::EntryKind::Directory {
			return Err(VolumeError::Unsupported(
				"folders cannot be opened for editing".into(),
			));
		}
		let file_name = remote_path
			.file_name()
			.map(|n| n.to_os_string())
			.ok_or_else(|| VolumeError::InvalidData("cloud path has no file name".into()))?;
		let id = Uuid::new_v4();
		let folder = self.root.join(id.to_string());
		tokio::fs::create_dir_all(&folder).await?;
		let local_path = folder.join(file_name);
		backend.read_to_file(&remote_path, &local_path).await?;

		let info = CloudEditSessionInfo {
			id,
			path,
			local_path: local_path.to_string_lossy().into_owned(),
			state: CloudEditState::Synced,
			last_synced_at: Some(Utc::now()),
		};
		let state = Arc::new(Mutex::new(SessionState {
			info: info.clone(),
			baseline: snapshot(&local_path).await,
			remote_revision: metadata.revision,
		}));
		let task = tokio::spawn(watch(
			backend.clone(),
			remote_path.clone(),
			local_path,
			state.clone(),
		));
		self.sessions.lock().await.insert(
			id,
			EditSession {
				backend,
				remote_path,
				state,
				task,
			},
		);
		info!(session = %id, "Opened cloud file for editing");
		Ok(CloudEditOpen::Local { session: info })
	}

	/// Every open session.
	pub async fn list(&self) -> Vec<CloudEditSessionInfo> {
		let sessions = self.sessions.lock().await;
		let mut infos = Vec::with_capacity(sessions.len());
		for session in sessions.values() {
			infos.push(session.state.lock().await.info.clone());
		}
		infos.sort_by(|a, b| a.local_path.cmp(&b.local_path));
		infos
	}

	/// Closes a session: uploads pending changes, stops watching and deletes
	/// the local copy.
	///
	/// When the final upload fails the local copy is kept and the error is
	/// returned, so the user can retry instead of losing the edit.
	pub async fn close(&self, id: Uuid) -> Result<(), VolumeError> {
		let Some(session) = self.sessions.lock().await.remove(&id) else {
			return Ok(());
		};
		session.task.abort();
		let local_path = PathBuf::from(&session.state.lock().await.info.local_path);
		if let Err(error) = sync_once(
			session.backend.as_ref(),
			&session.remote_path,
			&local_path,
			&session.state,
			true,
		)
		.await
		{
			let restart = tokio::spawn(watch(
				session.backend.clone(),
				session.remote_path.clone(),
				local_path,
				session.state.clone(),
			));
			self.sessions.lock().await.insert(
				id,
				EditSession {
					task: restart,
					..session
				},
			);
			return Err(error);
		}
		if let Some(folder) = local_path.parent() {
			let _ = tokio::fs::remove_dir_all(folder).await;
		}
		info!(session = %id, "Closed cloud editing session");
		Ok(())
	}
}

async fn watch(
	backend: Arc<dyn VolumeBackend>,
	remote_path: PathBuf,
	local_path: PathBuf,
	state: Arc<Mutex<SessionState>>,
) {
	let mut interval = tokio::time::interval(POLL_INTERVAL);
	interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
	loop {
		interval.tick().await;
		if let Err(error) =
			sync_once(backend.as_ref(), &remote_path, &local_path, &state, false).await
		{
			warn!(%error, "Could not upload edited cloud file; retrying on the next change");
			let mut state = state.lock().await;
			state.info.state = CloudEditState::Error {
				message: error.to_string(),
			};
		}
	}
}

/// Uploads the local copy if it changed since the last upload.
///
/// Without `force`, a change is uploaded only once the file has stayed the
/// same for `SETTLE_TIME`.
async fn sync_once(
	backend: &dyn VolumeBackend,
	remote_path: &Path,
	local_path: &Path,
	state: &Arc<Mutex<SessionState>>,
	force: bool,
) -> Result<(), VolumeError> {
	let Some(current) = snapshot(local_path).await else {
		// The app may be mid-rename; the next poll sees the new file.
		return Ok(());
	};
	{
		let mut guard = state.lock().await;
		if guard.baseline == Some(current) {
			return Ok(());
		}
		if !force {
			let settled = current
				.modified
				.and_then(|m| m.elapsed().ok())
				.is_some_and(|age| age >= SETTLE_TIME);
			if !settled {
				guard.info.state = CloudEditState::Modified;
				return Ok(());
			}
		}
		guard.info.state = CloudEditState::Uploading;
	}

	let expected = state.lock().await.remote_revision.clone();
	let remote = backend.metadata(remote_path).await.ok();
	let remote_changed = match (&expected, remote.as_ref().and_then(|m| m.revision.clone())) {
		(Some(expected), Some(actual)) => *expected != actual,
		_ => false,
	};

	if remote_changed {
		let copy_path = conflict_copy_path(remote_path);
		debug!(path = %copy_path.display(), "Cloud file changed remotely; uploading a conflict copy");
		backend.write_from_file(&copy_path, local_path).await?;
		let mut guard = state.lock().await;
		guard.baseline = Some(current);
		guard.info.state = CloudEditState::Conflict {
			copy_name: copy_path
				.file_name()
				.map(|n| n.to_string_lossy().into_owned())
				.unwrap_or_default(),
		};
		guard.info.last_synced_at = Some(Utc::now());
		return Ok(());
	}

	backend.write_from_file(remote_path, local_path).await?;
	let revision = backend
		.metadata(remote_path)
		.await
		.ok()
		.and_then(|m| m.revision);
	let mut guard = state.lock().await;
	guard.baseline = Some(current);
	guard.remote_revision = revision;
	guard.info.state = CloudEditState::Synced;
	guard.info.last_synced_at = Some(Utc::now());
	debug!(path = %remote_path.display(), "Uploaded edited cloud file");
	Ok(())
}

/// "report.docx" becomes "report (conflict copy 2026-11-12 1530).docx".
pub fn conflict_copy_path(path: &Path) -> PathBuf {
	let stem = path
		.file_stem()
		.map(|s| s.to_string_lossy().into_owned())
		.unwrap_or_default();
	let extension = path
		.extension()
		.map(|e| format!(".{}", e.to_string_lossy()))
		.unwrap_or_default();
	let stamp = chrono::Local::now().format("%Y-%m-%d %H%M%S");
	path.with_file_name(format!("{stem} (conflict copy {stamp}){extension}"))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::volume::backend::LocalBackend;

	fn cloud_path(name: &str) -> WingPath {
		WingPath::Cloud {
			service: crate::volume::backend::CloudServiceType::GoogleDrive,
			identifier: "me@example.com".into(),
			path: name.into(),
		}
	}

	/// Pushes the file's mtime back so the settle check passes without sleeping.
	fn age(path: &Path) {
		let old = SystemTime::now() - Duration::from_secs(60);
		let file = std::fs::File::options().write(true).open(path).unwrap();
		file.set_modified(old).unwrap();
	}

	/// Stops the session's background watcher and returns its state, so the
	/// test is the only caller of `sync_once`.
	///
	/// Left running, the watcher's poll can upload the same edit at the same
	/// time as the test, and the test then reads the remote file while that
	/// second write has truncated it but not yet written it.
	async fn stop_watcher(service: &CloudEditService, id: Uuid) -> Arc<Mutex<SessionState>> {
		let sessions = service.sessions.lock().await;
		let session = &sessions[&id];
		session.task.abort();
		session.state.clone()
	}

	#[tokio::test]
	async fn edits_are_uploaded_after_they_settle_and_close_cleans_up() {
		let remote = tempfile::tempdir().unwrap();
		let cache = tempfile::tempdir().unwrap();
		std::fs::write(remote.path().join("notes.txt"), "v1").unwrap();
		let backend: Arc<dyn VolumeBackend> = Arc::new(LocalBackend::new(remote.path()));
		let service = CloudEditService::new(cache.path().to_path_buf());

		let CloudEditOpen::Local { session } = service
			.open(
				cloud_path("notes.txt"),
				backend.clone(),
				PathBuf::from("notes.txt"),
			)
			.await
			.unwrap()
		else {
			panic!("local file expected");
		};
		let local = PathBuf::from(&session.local_path);
		assert_eq!(std::fs::read_to_string(&local).unwrap(), "v1");

		// A second open returns the same session.
		let CloudEditOpen::Local { session: again } = service
			.open(
				cloud_path("notes.txt"),
				backend.clone(),
				PathBuf::from("notes.txt"),
			)
			.await
			.unwrap()
		else {
			panic!();
		};
		assert_eq!(again.id, session.id);

		let state = stop_watcher(&service, session.id).await;
		std::fs::write(&local, "v2 edited").unwrap();
		// Fresh edits wait for the settle time.
		sync_once(
			backend.as_ref(),
			Path::new("notes.txt"),
			&local,
			&state,
			false,
		)
		.await
		.unwrap();
		assert_eq!(state.lock().await.info.state, CloudEditState::Modified);
		assert_eq!(
			std::fs::read_to_string(remote.path().join("notes.txt")).unwrap(),
			"v1"
		);

		age(&local);
		sync_once(
			backend.as_ref(),
			Path::new("notes.txt"),
			&local,
			&state,
			false,
		)
		.await
		.unwrap();
		assert_eq!(state.lock().await.info.state, CloudEditState::Synced);
		assert_eq!(
			std::fs::read_to_string(remote.path().join("notes.txt")).unwrap(),
			"v2 edited"
		);

		std::fs::write(&local, "v3 on close").unwrap();
		service.close(session.id).await.unwrap();
		assert_eq!(
			std::fs::read_to_string(remote.path().join("notes.txt")).unwrap(),
			"v3 on close"
		);
		assert!(!local.exists());
		assert!(service.list().await.is_empty());
	}

	#[derive(Debug)]
	struct Revisioned(LocalBackend, std::sync::Mutex<String>);

	#[async_trait::async_trait]
	impl VolumeBackend for Revisioned {
		async fn read(&self, p: &Path) -> Result<bytes::Bytes, VolumeError> {
			self.0.read(p).await
		}
		async fn read_range(
			&self,
			p: &Path,
			r: std::ops::Range<u64>,
		) -> Result<bytes::Bytes, VolumeError> {
			self.0.read_range(p, r).await
		}
		async fn write(&self, p: &Path, d: bytes::Bytes) -> Result<(), VolumeError> {
			self.0.write(p, d).await
		}
		async fn read_dir(
			&self,
			p: &Path,
		) -> Result<Vec<crate::volume::backend::RawDirEntry>, VolumeError> {
			self.0.read_dir(p).await
		}
		async fn metadata(
			&self,
			p: &Path,
		) -> Result<crate::volume::backend::RawMetadata, VolumeError> {
			let mut m = self.0.metadata(p).await?;
			m.revision = Some(self.1.lock().unwrap().clone());
			Ok(m)
		}
		async fn exists(&self, p: &Path) -> Result<bool, VolumeError> {
			self.0.exists(p).await
		}
		async fn delete(&self, p: &Path) -> Result<(), VolumeError> {
			self.0.delete(p).await
		}
		async fn create_directory(&self, p: &Path, r: bool) -> Result<(), VolumeError> {
			self.0.create_directory(p, r).await
		}
		fn is_local(&self) -> bool {
			false
		}
		fn backend_type(&self) -> crate::volume::backend::BackendType {
			crate::volume::backend::BackendType::Local
		}
	}

	#[tokio::test]
	async fn remote_change_produces_a_conflict_copy() {
		let remote = tempfile::tempdir().unwrap();
		let cache = tempfile::tempdir().unwrap();
		std::fs::write(remote.path().join("plan.md"), "base").unwrap();
		let backend = Arc::new(Revisioned(
			LocalBackend::new(remote.path()),
			std::sync::Mutex::new("r1".into()),
		));
		let service = CloudEditService::new(cache.path().to_path_buf());
		let CloudEditOpen::Local { session } = service
			.open(
				cloud_path("plan.md"),
				backend.clone(),
				PathBuf::from("plan.md"),
			)
			.await
			.unwrap()
		else {
			panic!();
		};
		let local = PathBuf::from(&session.local_path);
		let state = stop_watcher(&service, session.id).await;
		*backend.1.lock().unwrap() = "r2".into();
		std::fs::write(remote.path().join("plan.md"), "theirs").unwrap();
		std::fs::write(&local, "mine").unwrap();
		age(&local);
		sync_once(
			backend.as_ref(),
			Path::new("plan.md"),
			&local,
			&state,
			false,
		)
		.await
		.unwrap();
		let CloudEditState::Conflict { copy_name } = state.lock().await.info.state.clone() else {
			panic!("conflict expected");
		};
		assert!(copy_name.starts_with("plan (conflict copy "));
		assert!(copy_name.ends_with(".md"));
		assert_eq!(
			std::fs::read_to_string(remote.path().join("plan.md")).unwrap(),
			"theirs"
		);
		assert_eq!(
			std::fs::read_to_string(remote.path().join(&copy_name)).unwrap(),
			"mine"
		);
	}

	#[tokio::test]
	async fn preview_copies_are_cached_by_revision() {
		let remote = tempfile::tempdir().unwrap();
		let cache = tempfile::tempdir().unwrap();
		std::fs::write(remote.path().join("a.txt"), "one").unwrap();
		let backend = Revisioned(
			LocalBackend::new(remote.path()),
			std::sync::Mutex::new("r1".into()),
		);
		let service = CloudEditService::new(cache.path().to_path_buf());
		let first = service
			.preview_copy(&cloud_path("a.txt"), &backend, Path::new("a.txt"))
			.await
			.unwrap();
		assert_eq!(std::fs::read_to_string(&first).unwrap(), "one");
		std::fs::write(remote.path().join("a.txt"), "two").unwrap();
		let cached = service
			.preview_copy(&cloud_path("a.txt"), &backend, Path::new("a.txt"))
			.await
			.unwrap();
		assert_eq!(cached, first, "same revision reuses the download");
		*backend.1.lock().unwrap() = "r2".into();
		let fresh = service
			.preview_copy(&cloud_path("a.txt"), &backend, Path::new("a.txt"))
			.await
			.unwrap();
		assert_ne!(fresh, first);
		assert_eq!(std::fs::read_to_string(&fresh).unwrap(), "two");
		assert!(
			service.list().await.is_empty(),
			"previews are not editing sessions"
		);
	}

	#[test]
	fn conflict_copy_keeps_folder_and_extension() {
		let copy = conflict_copy_path(Path::new("docs/report.final.pdf"));
		assert_eq!(copy.parent(), Some(Path::new("docs")));
		let name = copy.file_name().unwrap().to_string_lossy().into_owned();
		assert!(name.starts_with("report.final (conflict copy "));
		assert!(name.ends_with(").pdf"));
	}
}
