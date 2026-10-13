//! Compare two local folders by relative path.
//!
//! Files on both sides are equal when their sizes and modification times
//! match, as rsync assumes by default; copies made with preserved timestamps
//! compare equal without being read. When only the times differ, a full blake3 read decides, so a merge
//! never overwrites one file with another that merely looks the same.

use crate::{
	context::CoreContext,
	infra::{
		api::SessionContext,
		query::{CoreQuery, QueryError, QueryResult},
	},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
	sync::Arc,
	time::UNIX_EPOCH,
};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CompareFoldersInput {
	pub left: PathBuf,
	pub right: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum CompareStatus {
	OnlyLeft,
	OnlyRight,
	/// Present on both sides with different content or type
	Different,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct CompareSide {
	pub is_dir: bool,
	/// Bytes; zero for folders
	pub size: u64,
	/// Seconds since the Unix epoch
	pub modified: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CompareEntry {
	/// Path below both roots, `/`-separated
	pub relative_path: String,
	pub status: CompareStatus,
	pub left: Option<CompareSide>,
	pub right: Option<CompareSide>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CompareFoldersOutput {
	/// Differences only; a folder present on one side is one entry
	pub entries: Vec<CompareEntry>,
	pub identical: u32,
	/// Entries that could not be read and were left out
	pub unreadable: u32,
}

struct Meta {
	side: CompareSide,
	modified_nanos: Option<u128>,
}

fn list(dir: &Path, unreadable: &mut u32) -> BTreeMap<String, Meta> {
	let mut entries = BTreeMap::new();
	let Ok(reader) = std::fs::read_dir(dir) else {
		*unreadable += 1;
		return entries;
	};
	for entry in reader.flatten() {
		// Symlinks are skipped so a merge never copies through one.
		let Ok(metadata) = std::fs::symlink_metadata(entry.path()) else {
			*unreadable += 1;
			continue;
		};
		if metadata.file_type().is_symlink() {
			continue;
		}
		let since_epoch = metadata
			.modified()
			.ok()
			.and_then(|t| t.duration_since(UNIX_EPOCH).ok());
		entries.insert(
			entry.file_name().to_string_lossy().into_owned(),
			Meta {
				side: CompareSide {
					is_dir: metadata.is_dir(),
					size: if metadata.is_dir() { 0 } else { metadata.len() },
					modified: since_epoch.map(|d| d.as_secs() as i64),
				},
				modified_nanos: since_epoch.map(|d| d.as_nanos()),
			},
		);
	}
	entries
}

fn same_bytes(a: &Path, b: &Path) -> std::io::Result<bool> {
	let hash = |path: &Path| -> std::io::Result<blake3::Hash> {
		let mut hasher = blake3::Hasher::new();
		hasher.update_reader(std::fs::File::open(path)?)?;
		Ok(hasher.finalize())
	};
	Ok(hash(a)? == hash(b)?)
}

fn walk(left: &Path, right: &Path, prefix: &str, out: &mut CompareFoldersOutput) {
	let l = list(left, &mut out.unreadable);
	let r = list(right, &mut out.unreadable);
	let mut names: Vec<&String> = l.keys().chain(r.keys()).collect();
	names.sort();
	names.dedup();

	for name in names {
		let relative_path = format!("{prefix}{name}");
		let entry = |status, left: Option<&Meta>, right: Option<&Meta>| CompareEntry {
			relative_path: relative_path.clone(),
			status,
			left: left.map(|m| m.side.clone()),
			right: right.map(|m| m.side.clone()),
		};
		match (l.get(name), r.get(name)) {
			(Some(a), None) => out
				.entries
				.push(entry(CompareStatus::OnlyLeft, Some(a), None)),
			(None, Some(b)) => out
				.entries
				.push(entry(CompareStatus::OnlyRight, None, Some(b))),
			(Some(a), Some(b)) if a.side.is_dir && b.side.is_dir => walk(
				&left.join(name),
				&right.join(name),
				&format!("{relative_path}/"),
				out,
			),
			(Some(a), Some(b)) => {
				let equal = !a.side.is_dir
					&& !b.side.is_dir
					&& a.side.size == b.side.size
					&& (a.modified_nanos == b.modified_nanos
						|| match same_bytes(&left.join(name), &right.join(name)) {
							Ok(same) => same,
							Err(_) => {
								out.unreadable += 1;
								continue;
							}
						});
				if equal {
					out.identical += 1;
				} else {
					out.entries
						.push(entry(CompareStatus::Different, Some(a), Some(b)));
				}
			}
			(None, None) => unreachable!("name came from one of the listings"),
		}
	}
}

pub fn compare_folders(left: &Path, right: &Path) -> Result<CompareFoldersOutput, String> {
	let left = left
		.canonicalize()
		.map_err(|e| format!("{}: {e}", left.display()))?;
	let right = right
		.canonicalize()
		.map_err(|e| format!("{}: {e}", right.display()))?;
	if !left.is_dir() || !right.is_dir() {
		return Err("both sides must be folders".to_string());
	}
	if left.starts_with(&right) || right.starts_with(&left) {
		return Err("one folder contains the other".to_string());
	}
	let mut out = CompareFoldersOutput {
		entries: Vec::new(),
		identical: 0,
		unreadable: 0,
	};
	walk(&left, &right, "", &mut out);
	Ok(out)
}

pub struct CompareFoldersQuery {
	input: CompareFoldersInput,
}

impl CoreQuery for CompareFoldersQuery {
	type Input = CompareFoldersInput;
	type Output = CompareFoldersOutput;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		if !input.left.is_absolute() || !input.right.is_absolute() {
			return Err(QueryError::InvalidInput(
				"folders must be absolute paths".to_string(),
			));
		}
		Ok(Self { input })
	}

	async fn execute(
		self,
		_context: Arc<CoreContext>,
		_session: SessionContext,
	) -> QueryResult<Self::Output> {
		let CompareFoldersInput { left, right } = self.input;
		tokio::task::spawn_blocking(move || compare_folders(&left, &right))
			.await
			.map_err(|e| QueryError::Internal(e.to_string()))?
			.map_err(QueryError::InvalidInput)
	}
}

crate::register_core_query!(CompareFoldersQuery, "files.compare_folders");

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reports_one_sided_and_changed_entries() {
		let root = tempfile::tempdir().unwrap();
		let (a, b) = (root.path().join("a"), root.path().join("b"));
		for side in [&a, &b] {
			std::fs::create_dir_all(side.join("shared")).unwrap();
			std::fs::write(side.join("shared/same.txt"), b"same").unwrap();
		}
		std::fs::write(a.join("shared/changed.txt"), b"left").unwrap();
		std::fs::write(b.join("shared/changed.txt"), b"rght").unwrap();
		// An edit moves the timestamp; same size and time would be taken as equal.
		std::fs::File::options()
			.write(true)
			.open(b.join("shared/changed.txt"))
			.unwrap()
			.set_modified(UNIX_EPOCH + std::time::Duration::from_secs(1_000_000))
			.unwrap();
		std::fs::create_dir_all(a.join("only_a_dir/deep")).unwrap();
		std::fs::write(b.join("only_b.txt"), b"b").unwrap();

		let out = compare_folders(&a, &b).unwrap();
		let summary: Vec<_> = out
			.entries
			.iter()
			.map(|e| (e.relative_path.as_str(), e.status))
			.collect();
		assert_eq!(
			summary,
			[
				("only_a_dir", CompareStatus::OnlyLeft),
				("only_b.txt", CompareStatus::OnlyRight),
				("shared/changed.txt", CompareStatus::Different),
			]
		);
		assert_eq!(out.identical, 1);
		assert!(compare_folders(&a, &a.join("shared")).is_err());
	}
}
