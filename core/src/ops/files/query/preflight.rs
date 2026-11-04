//! # Preflight plan for copy, move and delete
//!
//! Before a copy, move or delete starts, `files.preflight` answers what it would
//! do, without touching the disk: how many files and bytes, where each source
//! lands and what the conflict policy does there, and, for a delete, how many
//! files hold the last copy of their content in the library.
//!
//! The per-source rules come from [`crate::ops::files::plan`], which the copy job
//! runs too, so the plan and the job cannot disagree. The walk that counts files
//! stops after [`MAX_WALK_ENTRIES`] entries and reports `truncated`, so a huge
//! tree still answers in time.
//!
//! "Last copy" reads content identity from the index: a file whose content
//! appears nowhere else in the library. Files the index does not know, or knows
//! without a content identity, are counted as `unknown_files`, never as safe.

use std::{
	collections::{HashMap, HashSet},
	path::{Path, PathBuf},
	sync::Arc,
};

use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
	context::CoreContext,
	domain::addressing::WingPath,
	infra::query::{LibraryQuery, QueryError, QueryResult},
	ops::files::{
		copy::action::FileConflictResolution,
		plan::{self, PlanDecision},
	},
};

/// Entries the file walk visits before it gives up and marks the plan truncated.
pub const MAX_WALK_ENTRIES: usize = 200_000;
/// Per-source rows returned; totals still cover every source.
const MAX_ITEMS: usize = 1_000;
/// SQLite variables per `IN (...)` chunk.
const SQL_CHUNK: usize = 500;

/// The operation to plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum PreflightOperation {
	Copy,
	Move,
	Delete,
}

/// Input for `files.preflight`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FilePreflightInput {
	pub operation: PreflightOperation,
	pub sources: Vec<WingPath>,
	/// Folder (or new name) for a copy or move; ignored for a delete.
	pub destination: Option<WingPath>,
	/// Conflict policy the job will run with; `None` plans conflicts as `Ask`.
	pub on_conflict: Option<FileConflictResolution>,
}

/// What happens to one source.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PreflightItem {
	pub source: WingPath,
	/// Where a copy or move puts it; `None` for a delete.
	pub destination: Option<WingPath>,
	/// The conflict decision for a copy or move; `None` for a delete.
	pub decision: Option<PlanDecision>,
	pub files: u64,
	pub bytes: u64,
}

/// The plan: totals over every source, and the first sources in detail.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FilePreflightOutput {
	pub files: u64,
	pub bytes: u64,
	/// Sources whose destination is taken.
	pub conflicts: u64,
	/// Up to 1000 sources, in input order.
	pub items: Vec<PreflightItem>,
	/// The file walk stopped early; counts are lower bounds.
	pub truncated: bool,
	/// Delete only: indexed files whose content exists nowhere else in the library.
	pub last_copies: Option<u64>,
	/// Delete only: files the index cannot vouch for (not indexed, or no content identity).
	pub unknown_files: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FilePreflightQuery {
	pub input: FilePreflightInput,
}

impl LibraryQuery for FilePreflightQuery {
	type Input = FilePreflightInput;
	type Output = FilePreflightOutput;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		if input.sources.is_empty() {
			return Err(QueryError::InvalidInput("No sources to plan".into()));
		}
		if input.operation != PreflightOperation::Delete && input.destination.is_none() {
			return Err(QueryError::InvalidInput(
				"Copy and move need a destination".into(),
			));
		}
		Ok(Self { input })
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let input = self.input;
		let mut walk = Walk::default();
		let mut items = Vec::new();
		let mut conflicts = 0;

		for source in &input.sources {
			let (files, bytes) = match source.as_local_path() {
				Some(path) => walk.count(path).await,
				None => (1, 0),
			};

			let (destination, decision) = match (&input.operation, &input.destination) {
				(PreflightOperation::Delete, _) | (_, None) => (None, None),
				(_, Some(destination)) => {
					let target =
						plan::final_destination(source, destination, input.sources.len()).await;
					let decision = plan::decide(&target, input.on_conflict)
						.await
						.map_err(|error| QueryError::io_error(target.display(), error))?;
					if decision != PlanDecision::Write {
						conflicts += 1;
					}
					(Some(target), Some(decision))
				}
			};

			if items.len() < MAX_ITEMS {
				items.push(PreflightItem {
					source: source.clone(),
					destination,
					decision,
					files,
					bytes,
				});
			}
		}

		let (last_copies, unknown_files) = if input.operation == PreflightOperation::Delete {
			let library_id = session
				.current_library_id
				.ok_or_else(|| QueryError::Internal("No library in session".into()))?;
			let library = context
				.libraries()
				.await
				.get_library(library_id)
				.await
				.ok_or_else(|| QueryError::Internal("Library not found".into()))?;
			let local: Vec<PathBuf> = input
				.sources
				.iter()
				.filter_map(|source| source.as_local_path().map(Path::to_path_buf))
				.collect();
			let census = last_copy_census(library.db().conn(), &local).await?;
			let non_local = input
				.sources
				.iter()
				.filter(|source| source.as_local_path().is_none())
				.count() as u64;
			// Files on disk the index did not account for are unknown too.
			let unaccounted = walk.files.saturating_sub(census.indexed_files + non_local);
			(
				Some(census.last_copies),
				Some(census.without_content + unaccounted + non_local),
			)
		} else {
			(None, None)
		};

		Ok(FilePreflightOutput {
			files: walk.files,
			bytes: walk.bytes,
			conflicts,
			items,
			truncated: walk.truncated,
			last_copies,
			unknown_files,
		})
	}
}

/// Bounded recursive count of files and bytes, shared across all sources.
#[derive(Default)]
struct Walk {
	visited: usize,
	files: u64,
	bytes: u64,
	truncated: bool,
}

impl Walk {
	/// Counts `path` (a file or a whole folder) and returns its own totals.
	async fn count(&mut self, path: &Path) -> (u64, u64) {
		let (start_files, start_bytes) = (self.files, self.bytes);
		let mut stack = vec![path.to_path_buf()];
		while let Some(current) = stack.pop() {
			if self.visited >= MAX_WALK_ENTRIES {
				self.truncated = true;
				break;
			}
			self.visited += 1;
			// A link counts as one entry and is never followed, as copy and delete treat it.
			let Ok(metadata) = tokio::fs::symlink_metadata(&current).await else {
				continue;
			};
			if metadata.is_dir() {
				if let Ok(mut dir) = tokio::fs::read_dir(&current).await {
					while let Ok(Some(entry)) = dir.next_entry().await {
						stack.push(entry.path());
					}
				}
			} else {
				self.files += 1;
				self.bytes += metadata.len();
			}
		}
		(self.files - start_files, self.bytes - start_bytes)
	}
}

/// What the index knows about the files under the delete targets.
#[derive(Debug, Default, PartialEq, Eq)]
struct Census {
	/// Indexed files under the targets, with or without content identity.
	indexed_files: u64,
	/// Indexed files whose content appears nowhere outside the targets.
	last_copies: u64,
	/// Indexed files without a content identity.
	without_content: u64,
}

/// Counts the indexed files under `targets` and how many of them hold the last
/// copy of their content in the library.
async fn last_copy_census(db: &DatabaseConnection, targets: &[PathBuf]) -> QueryResult<Census> {
	let backend = db.get_database_backend();
	let full_name = "(e.name || CASE WHEN e.extension IS NULL OR e.extension = '' \
		THEN '' ELSE '.' || e.extension END)";

	// (entry id, content id) of every indexed file under the targets.
	let mut files: HashMap<i32, Option<i32>> = HashMap::new();
	for target in targets {
		let path = target.to_string_lossy();
		let (exact, below) = crate::ops::search::filters::directory_subtree_patterns(&path);
		let in_folder = Statement::from_sql_and_values(
			backend,
			"SELECT e.id AS id, e.content_id AS content_id FROM entries e \
			 JOIN directory_paths dp ON dp.entry_id = e.parent_id \
			 WHERE e.kind = 0 AND (dp.path = ? OR dp.path LIKE ? ESCAPE '\\')",
			[exact.clone().into(), below.into()],
		);
		let single = Statement::from_sql_and_values(
			backend,
			format!(
				"SELECT e.id AS id, e.content_id AS content_id FROM entries e \
				 JOIN directory_paths dp ON dp.entry_id = e.parent_id \
				 WHERE e.kind = 0 AND dp.path = ? AND {full_name} = ?"
			),
			[
				target
					.parent()
					.map(|parent| parent.to_string_lossy().into_owned())
					.unwrap_or_default()
					.into(),
				target
					.file_name()
					.map(|name| name.to_string_lossy().into_owned())
					.unwrap_or_default()
					.into(),
			],
		);
		for statement in [in_folder, single] {
			for row in db.query_all_raw(statement).await? {
				let id: i32 = row.try_get("", "id")?;
				let content_id: Option<i32> = row.try_get("", "content_id")?;
				files.insert(id, content_id);
			}
		}
	}

	let mut census = Census {
		indexed_files: files.len() as u64,
		..Census::default()
	};
	let mut targeted: HashMap<i32, u64> = HashMap::new();
	for content_id in files.values() {
		match content_id {
			Some(id) => *targeted.entry(*id).or_default() += 1,
			None => census.without_content += 1,
		}
	}

	let ids: Vec<i32> = targeted
		.keys()
		.copied()
		.collect::<HashSet<_>>()
		.into_iter()
		.collect();
	for chunk in ids.chunks(SQL_CHUNK) {
		let placeholders = vec!["?"; chunk.len()].join(",");
		let statement = Statement::from_sql_and_values(
			backend,
			format!(
				"SELECT content_id, COUNT(*) AS copies FROM entries \
				 WHERE kind = 0 AND content_id IN ({placeholders}) GROUP BY content_id"
			),
			chunk
				.iter()
				.map(|id| (*id).into())
				.collect::<Vec<sea_orm::Value>>(),
		);
		for row in db.query_all_raw(statement).await? {
			let content_id: i32 = row.try_get("", "content_id")?;
			let copies: i64 = row.try_get("", "copies")?;
			let doomed = targeted.get(&content_id).copied().unwrap_or(0);
			// Every copy of this content is being deleted.
			if copies as u64 <= doomed {
				census.last_copies += doomed;
			}
		}
	}

	Ok(census)
}

crate::register_library_query!(FilePreflightQuery, "files.preflight");

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn walk_counts_files_and_bytes_once_per_entry() {
		let dir = tempfile::tempdir().unwrap();
		std::fs::create_dir_all(dir.path().join("a/b")).unwrap();
		std::fs::write(dir.path().join("a/one.txt"), "12345").unwrap();
		std::fs::write(dir.path().join("a/b/two.txt"), "12").unwrap();
		std::fs::write(dir.path().join("three.txt"), "1").unwrap();

		let mut walk = Walk::default();
		assert_eq!(walk.count(&dir.path().join("a")).await, (2, 7));
		assert_eq!(walk.count(&dir.path().join("three.txt")).await, (1, 1));
		assert_eq!((walk.files, walk.bytes, walk.truncated), (3, 8, false));
	}
}
