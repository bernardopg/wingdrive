//! Path diff action.

use super::{
	input::{DiffStrategy, PathDiffInput},
	output::{CompareStatus, DiffEntry, PathDiffResult},
};
use crate::{
	context::CoreContext,
	domain::{addressing::WingPath, file::EntryKind},
	infra::{
		action::{error::ActionError, LibraryAction, ValidationResult},
		job::manager::JobManager,
	},
	ops::indexing::{
		ephemeral::{EphemeralIndex, EphemeralIndexCache},
		IndexScope, IndexerJob, IndexerJobConfig,
	},
};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::Arc,
	time::UNIX_EPOCH,
};

#[derive(Debug, Clone)]
pub struct PathDiffAction {
	input: PathDiffInput,
}

impl LibraryAction for PathDiffAction {
	type Input = PathDiffInput;
	type Output = PathDiffResult;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	fn action_kind(&self) -> &'static str {
		"files.diff"
	}

	async fn validate(
		&self,
		_library: &Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<ValidationResult, ActionError> {
		for (field, path) in [
			("source", &self.input.source),
			("target", &self.input.target),
		] {
			match path {
				WingPath::Physical { .. } if path.as_local_path().is_some() => {}
				_ => {
					return Err(ActionError::Validation {
						field: field.to_string(),
						message: "Only local physical paths are supported for diff".to_string(),
					});
				}
			}
		}
		Ok(ValidationResult::Success { metadata: None })
	}

	async fn execute(
		self,
		library: Arc<crate::library::Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let source_path = local_path(&self.input.source, "source")?;
		let target_path = local_path(&self.input.target, "target")?;

		ensure_indexed(
			&self.input.source,
			&source_path,
			self.input.use_index_rules,
			&library,
			&context,
		)
		.await?;
		ensure_indexed(
			&self.input.target,
			&target_path,
			self.input.use_index_rules,
			&library,
			&context,
		)
		.await?;

		let cache = context.ephemeral_cache();
		let source_map = build_path_map(&source_path, &self.input.source, cache).await?;
		let target_map = build_path_map(&target_path, &self.input.target, cache).await?;

		let mut result = match self.input.strategy {
			DiffStrategy::Heuristic => diff_heuristic(&source_map, &target_map),
			DiffStrategy::Content => diff_content(&source_map, &target_map),
			DiffStrategy::Hybrid => diff_hybrid(&source_map, &target_map),
		};
		result.total_scanned = source_map.len().max(target_map.len());
		Ok(result)
	}
}

crate::register_library_action!(PathDiffAction, "files.diff");

fn local_path(path: &WingPath, field: &str) -> Result<PathBuf, ActionError> {
	path.as_local_path()
		.map(|p| p.to_path_buf())
		.ok_or_else(|| ActionError::Validation {
			field: field.to_string(),
			message: "Only local physical paths are supported for diff".to_string(),
		})
}

async fn ensure_indexed(
	wing_path: &WingPath,
	local: &Path,
	use_index_rules: bool,
	library: &Arc<crate::library::Library>,
	context: &Arc<CoreContext>,
) -> Result<(), ActionError> {
	let cache = context.ephemeral_cache();
	if cache.is_indexed(local) {
		return Ok(());
	}

	let config = if use_index_rules {
		IndexerJobConfig::ephemeral_browse(wing_path.clone(), IndexScope::Recursive, false)
	} else {
		IndexerJobConfig::complete_scan(wing_path.clone(), IndexScope::Recursive)
	};

	let mut job = IndexerJob::new(config);
	job.set_ephemeral_index(cache.get_global_index());

	let handle = library
		.jobs()
		.dispatch(job)
		.await
		.map_err(|e| ActionError::Internal(format!("Failed to dispatch indexer: {e}")))?;
	handle
		.wait()
		.await
		.map_err(|e| ActionError::Internal(format!("Indexer failed: {e}")))?;

	Ok(())
}

async fn build_path_map(
	root: &Path,
	root_wing: &WingPath,
	cache: &Arc<EphemeralIndexCache>,
) -> Result<HashMap<PathBuf, DiffEntry>, ActionError> {
	let index = cache
		.get_for_path(root)
		.ok_or_else(|| ActionError::Internal(format!("Path not indexed: {}", root.display())))?;
	let index_read = index.read().await;
	let entries = index_read.entries();
	drop(index_read);

	let mut map = HashMap::new();
	for (abs_path, meta) in entries {
		let Ok(relative) = abs_path.strip_prefix(root) else {
			continue;
		};
		if relative.as_os_str().is_empty() {
			continue;
		}
		let modified_at = meta
			.modified
			.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
			.map(|d| d.as_secs() as i64);
		let wing_path = match root_wing {
			WingPath::Physical { device_slug, .. } => {
				WingPath::physical(device_slug.clone(), abs_path.clone())
			}
			_ => WingPath::local(abs_path.clone()),
		};
		let kind = match meta.kind {
			crate::ops::indexing::state::EntryKind::File => EntryKind::File,
			crate::ops::indexing::state::EntryKind::Directory => EntryKind::Directory,
			crate::ops::indexing::state::EntryKind::Symlink => EntryKind::Symlink,
		};
		map.insert(
			relative.to_path_buf(),
			DiffEntry {
				relative_path: relative.to_path_buf(),
				wing_path,
				uuid: None,
				size: meta.size,
				modified_at,
				content_id: None,
				kind,
			},
		);
	}
	Ok(map)
}

fn diff_heuristic(
	source: &HashMap<PathBuf, DiffEntry>,
	target: &HashMap<PathBuf, DiffEntry>,
) -> PathDiffResult {
	let mut result = PathDiffResult::default();
	for (rel, s) in source {
		match target.get(rel) {
			None => result.only_in_source.push(s.clone()),
			Some(t) => {
				if s.kind != t.kind || s.size != t.size || s.modified_at != t.modified_at {
					result.modified.push(s.clone());
				} else {
					result.matched_count += 1;
				}
			}
		}
	}
	for (rel, t) in target {
		if !source.contains_key(rel) {
			result.only_in_target.push(t.clone());
		}
	}
	result.copy_size = result
		.only_in_source
		.iter()
		.chain(result.modified.iter())
		.filter(|e| e.kind == EntryKind::File)
		.map(|e| e.size)
		.sum();
	result
}

fn diff_content(
	source: &HashMap<PathBuf, DiffEntry>,
	target: &HashMap<PathBuf, DiffEntry>,
) -> PathDiffResult {
	// Without content IDs, fall back to heuristic for correctness.
	// Content IDs can be added when persistent/content-indexed paths are supported.
	let mut result = diff_heuristic(source, target);
	// Mark as content strategy: if content_id exists, compare by it.
	// For now, heuristic result is the safe baseline.
	result
}

fn diff_hybrid(
	source: &HashMap<PathBuf, DiffEntry>,
	target: &HashMap<PathBuf, DiffEntry>,
) -> PathDiffResult {
	// Hybrid starts with heuristic; content verification requires content hashes.
	diff_heuristic(source, target)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::domain::file::EntryKind;

	fn entry(rel: &str, size: u64, modified_at: Option<i64>) -> DiffEntry {
		DiffEntry {
			relative_path: PathBuf::from(rel),
			wing_path: WingPath::local(PathBuf::from(format!("/tmp/{rel}"))),
			uuid: None,
			size,
			modified_at,
			content_id: None,
			kind: EntryKind::File,
		}
	}

	#[test]
	fn heuristic_detects_missing_extra_modified_and_matched() {
		let mut source = HashMap::new();
		source.insert(PathBuf::from("a.txt"), entry("a.txt", 10, Some(1)));
		source.insert(PathBuf::from("b.txt"), entry("b.txt", 10, Some(1)));
		source.insert(PathBuf::from("c.txt"), entry("c.txt", 20, Some(2)));

		let mut target = HashMap::new();
		target.insert(PathBuf::from("a.txt"), entry("a.txt", 10, Some(1)));
		target.insert(PathBuf::from("b.txt"), entry("b.txt", 10, Some(99)));
		target.insert(PathBuf::from("d.txt"), entry("d.txt", 5, Some(1)));

		let result = diff_heuristic(&source, &target);
		assert_eq!(result.matched_count, 1);
		assert_eq!(result.only_in_source.len(), 1);
		assert_eq!(
			result.only_in_source[0].relative_path,
			PathBuf::from("c.txt")
		);
		assert_eq!(result.only_in_target.len(), 1);
		assert_eq!(
			result.only_in_target[0].relative_path,
			PathBuf::from("d.txt")
		);
		assert_eq!(result.modified.len(), 1);
		assert_eq!(result.modified[0].relative_path, PathBuf::from("b.txt"));
		assert_eq!(result.copy_size, 30);
	}
}
