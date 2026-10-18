//! Browsing archives as folders.
//!
//! A path below an archive file (`/home/a/photos.zip/2024/x.jpg`) names a
//! member of that archive. Listings read the archive's index without
//! extracting, and opening a member extracts just that file into a cache.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use super::engine::{self, ArchiveError, Control, Member};
use super::format::ArchiveFormat;
use crate::context::CoreContext;
use crate::domain::addressing::WingPath;
use crate::infra::action::{error::ActionError, LibraryAction};

/// Splits `path` into the archive file it lies in and the path inside it
/// (empty for the archive itself). `None` for real folders and plain files.
pub fn split(path: &Path) -> Option<(PathBuf, PathBuf)> {
	for ancestor in path.ancestors() {
		match std::fs::metadata(ancestor) {
			Ok(meta) if meta.is_file() => {
				ArchiveFormat::detect(ancestor)?;
				let inner = path.strip_prefix(ancestor).ok()?;
				if inner.as_os_str().is_empty() {
					return Some((ancestor.to_path_buf(), PathBuf::new()));
				}
				return Some((ancestor.to_path_buf(), engine::safe_relative(inner).ok()?));
			}
			// A real folder ends the search; a missing path may be inside an archive.
			Ok(_) => return None,
			Err(_) => continue,
		}
	}
	None
}

/// Size and modification time an archive had when it was listed.
type Stamp = (u64, Option<SystemTime>);

/// Archives listed recently, so browsing a compressed tar does not
/// decompress it again for every folder.
fn listings() -> &'static Mutex<HashMap<PathBuf, (Stamp, Arc<Vec<Member>>)>> {
	static LISTINGS: OnceLock<Mutex<HashMap<PathBuf, (Stamp, Arc<Vec<Member>>)>>> = OnceLock::new();
	LISTINGS.get_or_init(Default::default)
}

const LISTINGS_KEPT: usize = 16;

fn stamp(archive: &Path) -> Result<Stamp, ArchiveError> {
	let meta = std::fs::metadata(archive)?;
	Ok((meta.len(), meta.modified().ok()))
}

fn format_of(archive: &Path) -> Result<ArchiveFormat, ArchiveError> {
	ArchiveFormat::detect(archive)
		.ok_or_else(|| ArchiveError::Unsupported("Not a supported archive".into()))
}

/// Members of `archive`, from the cache while the file is unchanged.
pub fn members(archive: &Path) -> Result<Arc<Vec<Member>>, ArchiveError> {
	let stamp = stamp(archive)?;
	if let Some((cached, members)) = listings().lock().unwrap().get(archive) {
		if *cached == stamp {
			return Ok(members.clone());
		}
	}
	let members = Arc::new(engine::list(archive, format_of(archive)?)?);
	let mut listings = listings().lock().unwrap();
	// shortcut: drops every listing when full; an LRU only matters with many archives open.
	if listings.len() >= LISTINGS_KEPT {
		listings.clear();
	}
	listings.insert(archive.to_path_buf(), (stamp, members.clone()));
	Ok(members)
}

/// Extracts the file `inner` of `archive` into `cache_root` once and returns
/// its path there, so apps and previews can open it like any other file.
pub fn materialize(
	archive: &Path,
	inner: &Path,
	cache_root: &Path,
) -> Result<PathBuf, ArchiveError> {
	let inner = engine::safe_relative(inner)?;
	let (len, modified) = stamp(archive)?;
	let key = blake3::hash(
		format!(
			"{}\0{len}\0{modified:?}\0{}",
			archive.display(),
			inner.display()
		)
		.as_bytes(),
	);
	// One flat folder per member, and a name without a leading dot: the
	// webview's asset protocol refuses paths with hidden components.
	let root = cache_root.join(&key.to_hex()[..16]);
	let name = inner
		.file_name()
		.map(|n| n.to_string_lossy().into_owned())
		.unwrap_or_default();
	let target = root.join(match name.strip_prefix('.') {
		Some(rest) => format!("_{rest}"),
		None => name,
	});
	if target.is_file() {
		return Ok(target);
	}

	let temp = cache_root.join(format!("tmp-{}", Uuid::new_v4()));
	std::fs::create_dir_all(&temp)?;
	let result = (|| {
		engine::extract_only(
			archive,
			format_of(archive)?,
			&temp,
			Some(&inner),
			&Control::default(),
		)?;
		let extracted = temp.join(&inner);
		if !extracted.is_file() {
			return Err(ArchiveError::Unsupported(format!(
				"{} is not a file in this archive",
				inner.display()
			)));
		}
		std::fs::create_dir_all(&root)?;
		std::fs::rename(&extracted, &target)?;
		Ok(target)
	})();
	let _ = std::fs::remove_dir_all(&temp);
	result
}

/// Copies older than this are removed when the daemon first opens a member.
const CACHE_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(7 * 24 * 60 * 60);

/// Removes cached copies and leftover temp folders older than `max_age`.
pub fn prune(cache_root: &Path, max_age: std::time::Duration) {
	let Ok(entries) = std::fs::read_dir(cache_root) else {
		return;
	};
	for entry in entries.flatten() {
		let old = entry
			.metadata()
			.and_then(|m| m.modified())
			.ok()
			.and_then(|t| t.elapsed().ok())
			.is_some_and(|age| age > max_age);
		if old {
			if let Err(error) = std::fs::remove_dir_all(entry.path()) {
				tracing::warn!(path = %entry.path().display(), %error, "Could not prune archive cache");
			}
		}
	}
}

/// Folder that holds files opened from archives.
pub fn cache_root(context: &CoreContext) -> PathBuf {
	context.data_dir.join("cache").join("archives")
}

/// Input for opening a file that may lie inside an archive
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ArchiveOpenMemberInput {
	pub path: WingPath,
}

/// A path on disk that apps can open
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ArchiveOpenMemberOutput {
	/// The extracted copy for an archive member, otherwise the input path
	pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveOpenMemberAction {
	input: ArchiveOpenMemberInput,
}

impl LibraryAction for ArchiveOpenMemberAction {
	type Input = ArchiveOpenMemberInput;
	type Output = ArchiveOpenMemberOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		_library: Arc<crate::library::Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let path = super::job::local(&self.input.path, "path")?;
		let cache = cache_root(&context);
		static PRUNED: OnceLock<()> = OnceLock::new();
		let prune_first = PRUNED.set(()).is_ok();
		let opened = tokio::task::spawn_blocking(move || {
			if prune_first {
				prune(&cache, CACHE_MAX_AGE);
			}
			match split(&path) {
				Some((archive, inner)) if !inner.as_os_str().is_empty() => {
					materialize(&archive, &inner, &cache)
				}
				_ => Ok(path),
			}
		})
		.await
		.map_err(|e| ActionError::Internal(e.to_string()))?
		.map_err(|e| ActionError::Internal(e.to_string()))?;
		Ok(ArchiveOpenMemberOutput {
			path: opened.to_string_lossy().into_owned(),
		})
	}

	fn action_kind(&self) -> &'static str {
		"archive.open_member"
	}
}

crate::register_library_action!(ArchiveOpenMemberAction, "archive.open_member");

#[cfg(test)]
mod tests {
	use super::*;

	fn sample(dir: &Path) -> PathBuf {
		let docs = dir.join("docs");
		std::fs::create_dir_all(docs.join("deep")).unwrap();
		std::fs::write(docs.join("deep/note.txt"), "inside").unwrap();
		let archive = dir.join("pack.zip");
		engine::compress(&[docs], ArchiveFormat::Zip, &archive, &Control::default()).unwrap();
		archive
	}

	#[test]
	fn paths_below_an_archive_split_into_archive_and_member() {
		let dir = tempfile::tempdir().unwrap();
		let archive = sample(dir.path());
		assert_eq!(split(&archive), Some((archive.clone(), PathBuf::new())));
		assert_eq!(
			split(&archive.join("docs/deep")),
			Some((archive.clone(), PathBuf::from("docs/deep")))
		);
		assert_eq!(split(dir.path()), None);
		assert_eq!(split(&dir.path().join("docs/deep/note.txt")), None);
		assert_eq!(split(&archive.join("docs/../../escape")), None);
	}

	#[test]
	fn members_are_extracted_once_into_the_cache() {
		let dir = tempfile::tempdir().unwrap();
		let archive = sample(dir.path());
		let cache = dir.path().join("cache");
		let inner = Path::new("docs/deep/note.txt");
		let opened = materialize(&archive, inner, &cache).unwrap();
		assert!(opened.starts_with(&cache) && opened.ends_with("note.txt"));
		assert_eq!(std::fs::read_to_string(&opened).unwrap(), "inside");
		assert_eq!(materialize(&archive, inner, &cache).unwrap(), opened);
		assert!(materialize(&archive, Path::new("docs/deep"), &cache).is_err());
		// Nothing but the copy is left behind, and only stale copies are pruned.
		assert_eq!(std::fs::read_dir(&cache).unwrap().count(), 1);
		prune(&cache, std::time::Duration::from_secs(3600));
		assert!(opened.is_file());
		prune(&cache, std::time::Duration::ZERO);
		assert!(!opened.exists());
		assert_eq!(members(&archive).unwrap().len(), 3);
	}
}
