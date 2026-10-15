//! # Directory Size Aggregation
//!
//! Computes total sizes and file counts for directories by traversing from deepest
//! leaves to the root. Each directory's `aggregate_size` includes all descendant files,
//! and `file_count` tracks the total number of files (not subdirectories) contained
//! within. This data powers folder size displays in the UI and enables sorting by size.
//!
//! Processing order matters: children must be aggregated before their parents, so we
//! sort directories by depth (deepest first) before computing. Without this, parent
//! totals would miss unaggregated child contributions. The closure table provides all
//! descendants in one query instead of recursive tree walks.

use crate::{
	infra::{
		db::entities::{self, entry_closure},
		job::generic_progress::ToGenericProgress,
		job::prelude::{JobContext, JobError, Progress},
	},
	ops::indexing::state::{IndexPhase, IndexerProgress, IndexerState, Phase},
};
use sea_orm::{
	ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbBackend, DbErr,
	EntityTrait, FromQueryResult, QueryFilter, QueryOrder, Statement,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Aggregates directory sizes and file counts from leaves to root.
///
/// Queries all directories under the location using the closure table, sorts them by
/// depth (deepest first), then computes aggregate_size and file_count for each by
/// summing direct children. Updates indexed_at after each directory so sync picks up
/// the aggregated values. Skips locations without an entry_id (not yet indexed).
pub async fn run_aggregation_phase(
	location_id: Uuid,
	state: &mut IndexerState,
	ctx: &JobContext<'_>,
) -> Result<(), JobError> {
	ctx.log("Starting directory size aggregation phase");

	let location_record = entities::location::Entity::find()
		.filter(entities::location::Column::Uuid.eq(location_id))
		.one(ctx.library_db())
		.await
		.map_err(|e| JobError::execution(format!("Failed to find location: {}", e)))?
		.ok_or_else(|| JobError::execution("Location not found in database".to_string()))?;

	let location_id_i32 = location_record.id;

	let descendant_ids = entities::entry_closure::Entity::find()
		.filter(entities::entry_closure::Column::AncestorId.eq(location_record.entry_id))
		.all(ctx.library_db())
		.await
		.map_err(|e| JobError::execution(format!("Failed to query closure table: {}", e)))?
		.into_iter()
		.map(|ec| ec.descendant_id)
		.collect::<Vec<i32>>();

	let Some(root_entry_id) = location_record.entry_id else {
		return Ok(());
	};
	let mut all_entry_ids = vec![root_entry_id];
	all_entry_ids.extend(descendant_ids);

	let mut directories: Vec<entities::entry::Model> = Vec::new();
	// SQLite has a bind parameter limit (~999). Query in safe chunks.
	let chunk_size: usize = 900;
	for chunk in all_entry_ids.chunks(chunk_size) {
		let mut batch = entities::entry::Entity::find()
			.filter(entities::entry::Column::Id.is_in(chunk.to_vec()))
			.filter(entities::entry::Column::Kind.eq(1)) // Directory
			.all(ctx.library_db())
			.await
			.map_err(|e| JobError::execution(format!("Failed to query directories: {}", e)))?;
		directories.append(&mut batch);
	}

	let parents: HashMap<i32, Option<i32>> = directories
		.iter()
		.map(|directory| (directory.id, directory.parent_id))
		.collect();

	// Count depth by following parent links up to root.
	let mut dir_depths: Vec<(entities::entry::Model, usize)> = Vec::new();

	for directory in directories {
		let mut depth = 0;
		let mut current_parent_id = directory.parent_id;

		while let Some(parent_id) = current_parent_id {
			depth += 1;
			if let Ok(Some(parent)) = entities::entry::Entity::find_by_id(parent_id)
				.one(ctx.library_db())
				.await
			{
				current_parent_id = parent.parent_id;
			} else {
				break;
			}
		}

		dir_depths.push((directory, depth));
	}

	dir_depths.sort_by(|a, b| b.1.cmp(&a.1));
	let directories: Vec<entities::entry::Model> =
		dir_depths.into_iter().map(|(dir, _)| dir).collect();

	let total_dirs = directories.len();
	ctx.log(format!("Found {} directories to aggregate", total_dirs));

	let mut processed = 0;
	let overcounts = hard_link_overcounts(ctx.library_db(), root_entry_id, &parents)
		.await
		.map_err(|e| JobError::execution(format!("Failed to find hard links: {}", e)))?;
	let aggregator = DirectoryAggregator::new(ctx.library_db().clone(), overcounts);

	for directory in directories {
		ctx.check_interrupt().await?;

		processed += 1;
		let indexer_progress = IndexerProgress {
			phase: IndexPhase::Finalizing {
				processed: processed,
				total: total_dirs,
			},
			current_path: format!(
				"Aggregating directory {}/{}: {}",
				processed, total_dirs, directory.name
			),
			total_found: state.stats,
			processing_rate: state.calculate_rate(),
			estimated_remaining: state.estimate_remaining(),
			scope: None,
			persistence: None,
			is_ephemeral: false,
			action_context: None, // TODO: Pass action context from job state
			volume_total_capacity: state.volume_total_capacity,
		};
		ctx.progress(Progress::generic(indexer_progress.to_generic_progress()));

		match aggregator.aggregate_directory(&directory).await {
			Ok((aggregate_size, child_count, file_count)) => {
				let directory_name = directory.name.clone();
				let mut active_dir: entities::entry::ActiveModel = directory.into();
				active_dir.aggregate_size = Set(aggregate_size);
				active_dir.child_count = Set(child_count);
				active_dir.file_count = Set(file_count);
				// Bump indexed_at so sync picks up aggregate changes.
				active_dir.indexed_at = Set(Some(chrono::Utc::now()));

				active_dir.update(ctx.library_db()).await.map_err(|e| {
					JobError::execution(format!("Failed to update directory aggregates: {}", e))
				})?;

				ctx.log(format!(
					"Aggregated {}: {} bytes, {} children, {} files",
					directory_name, aggregate_size, child_count, file_count
				));
			}
			Err(e) => {
				ctx.add_non_critical_error(format!(
					"Failed to aggregate directory {}: {}",
					directory.name, e
				));
			}
		}
	}

	ctx.log(format!(
		"Directory aggregation complete: {} directories processed",
		processed
	));
	state.phase = Phase::ContentIdentification;
	Ok(())
}

/// What a directory would count more than once through hard links.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Overcount {
	bytes: i64,
	files: i32,
}

struct DirectoryAggregator {
	db: DatabaseConnection,
	/// Per-directory hard-link overcount; see [`hard_link_overcounts`].
	overcounts: HashMap<i32, Overcount>,
}

impl DirectoryAggregator {
	fn new(db: DatabaseConnection, overcounts: HashMap<i32, Overcount>) -> Self {
		Self { db, overcounts }
	}

	fn overcount(&self, directory_id: i32) -> Overcount {
		self.overcounts
			.get(&directory_id)
			.copied()
			.unwrap_or_default()
	}

	/// Computes aggregate values by summing direct children only.
	///
	/// Files contribute their size directly. Subdirectories contribute their already-computed
	/// aggregate_size and file_count (this is why we process deepest-first). Symlinks are
	/// treated as files for counting purposes.
	///
	/// A hard-linked file is stored once on disk, so the bytes and files its extra
	/// links add are subtracted. Stored child totals already had their own overcount removed,
	/// so it is added back before the parent's overcount, which covers links that
	/// span sibling subtrees, is subtracted once.
	async fn aggregate_directory(
		&self,
		directory: &entities::entry::Model,
	) -> Result<(i64, i32, i32), DbErr> {
		let children = entities::entry::Entity::find()
			.filter(entities::entry::Column::ParentId.eq(directory.id))
			.all(&self.db)
			.await?;

		let mut aggregate_size = 0i64;
		let child_count = children.len() as i32;
		let mut file_count = 0i32;

		for child in children {
			match child.kind {
				0 => {
					aggregate_size += child.size;
					file_count += 1;
				}
				1 => {
					let overcount = self.overcount(child.id);
					aggregate_size += child.aggregate_size + overcount.bytes;
					file_count += child.file_count + overcount.files;
				}
				2 => {
					// Symlinks count as files.
					aggregate_size += child.size;
					file_count += 1;
				}
				_ => {}
			}
		}

		let overcount = self.overcount(directory.id);
		aggregate_size -= overcount.bytes;
		file_count -= overcount.files;

		Ok((aggregate_size, child_count, file_count))
	}
}

#[derive(Debug, FromQueryResult)]
struct HardLinkedFile {
	parent_id: Option<i32>,
	volume_id: Option<i32>,
	inode: i64,
	size: i64,
}

/// Finds the bytes and files each directory counts more than once because
/// several file entries under it are hard links to one inode.
///
/// For every ancestor directory, a group of `n` links to the same inode inside
/// it adds `size * (n - 1)` extra bytes and `n - 1` extra files. Hard links are rare, so SQL only
/// returns files whose inode appears more than once under the location.
/// Ancestors are walked through `parent_id` (`parents` maps each directory of
/// the location to its parent) rather than the closure table, which can miss
/// the direct-parent row of entries inserted in the same batch as their parent.
async fn hard_link_overcounts(
	db: &DatabaseConnection,
	root_entry_id: i32,
	parents: &HashMap<i32, Option<i32>>,
) -> Result<HashMap<i32, Overcount>, DbErr> {
	let statement = Statement::from_sql_and_values(
		DbBackend::Sqlite,
		r#"
		WITH files AS (
			SELECT e.parent_id, e.volume_id, e.inode, e.size
			FROM entries e
			JOIN entry_closure scope ON scope.descendant_id = e.id
			WHERE scope.ancestor_id = ? AND e.kind = 0 AND e.inode IS NOT NULL
		),
		linked AS (
			SELECT volume_id, inode FROM files
			GROUP BY volume_id, inode
			HAVING COUNT(*) > 1
		)
		SELECT f.parent_id, f.volume_id, f.inode, f.size
		FROM files f
		JOIN linked l ON l.volume_id IS f.volume_id AND l.inode = f.inode
		"#,
		[root_entry_id.into()],
	);
	let files = HardLinkedFile::find_by_statement(statement).all(db).await?;

	// (directory, volume, inode) -> (size, links under that directory)
	let mut groups: HashMap<(i32, Option<i32>, i64), (i64, i64)> = HashMap::new();
	for file in files {
		let mut ancestor = file.parent_id;
		while let Some(directory) = ancestor.filter(|id| parents.contains_key(id)) {
			let group = groups
				.entry((directory, file.volume_id, file.inode))
				.or_insert((file.size, 0));
			group.0 = group.0.max(file.size);
			group.1 += 1;
			ancestor = parents[&directory];
		}
	}

	let mut overcounts: HashMap<i32, Overcount> = HashMap::new();
	for ((directory, _, _), (size, links)) in groups {
		if links > 1 {
			let overcount = overcounts.entry(directory).or_default();
			overcount.bytes += size * (links - 1);
			overcount.files += (links - 1) as i32;
		}
	}
	Ok(overcounts)
}

/// Backfills aggregate_size and file_count for all existing directories across all locations.
///
/// This is a one-time migration for databases created before aggregation was added.
/// Safe to run multiple times (idempotent). Processes each location independently,
/// sorting directories by depth within each location tree.
pub async fn migrate_directory_sizes(db: &DatabaseConnection) -> Result<(), DbErr> {
	let locations = entities::location::Entity::find().all(db).await?;

	for location in locations {
		tracing::info!(
			"Migrating directory sizes for location: {}",
			location.name.as_deref().unwrap_or("Unknown")
		);

		let Some(root_entry_id) = location.entry_id else {
			tracing::warn!(
				"Skipping location {} - entry_id not set (not yet synced)",
				location.name.as_deref().unwrap_or("Unknown")
			);
			continue;
		};

		let descendant_ids = entry_closure::Entity::find()
			.filter(entry_closure::Column::AncestorId.eq(root_entry_id))
			.all(db)
			.await?
			.into_iter()
			.map(|ec| ec.descendant_id)
			.collect::<Vec<i32>>();

		let mut all_entry_ids = vec![root_entry_id];
		all_entry_ids.extend(descendant_ids);

		let mut directories: Vec<entities::entry::Model> = Vec::new();
		let chunk_size: usize = 900;
		for chunk in all_entry_ids.chunks(chunk_size) {
			let mut batch = entities::entry::Entity::find()
				.filter(entities::entry::Column::Id.is_in(chunk.to_vec()))
				.filter(entities::entry::Column::Kind.eq(1))
				.all(db)
				.await?;
			directories.append(&mut batch);
		}

		let parents: HashMap<i32, Option<i32>> = directories
			.iter()
			.map(|directory| (directory.id, directory.parent_id))
			.collect();

		let mut dir_depths: Vec<(entities::entry::Model, usize)> = Vec::new();

		for directory in directories {
			let mut depth = 0;
			let mut current_parent_id = directory.parent_id;

			while let Some(parent_id) = current_parent_id {
				depth += 1;
				if let Ok(Some(parent)) =
					entities::entry::Entity::find_by_id(parent_id).one(db).await
				{
					current_parent_id = parent.parent_id;
				} else {
					break;
				}
			}

			dir_depths.push((directory, depth));
		}

		dir_depths.sort_by(|a, b| b.1.cmp(&a.1));
		let directories: Vec<entities::entry::Model> =
			dir_depths.into_iter().map(|(dir, _)| dir).collect();

		let overcounts = hard_link_overcounts(db, root_entry_id, &parents).await?;
		let aggregator = DirectoryAggregator::new(db.clone(), overcounts);

		for directory in directories {
			match aggregator.aggregate_directory(&directory).await {
				Ok((aggregate_size, child_count, file_count)) => {
					let mut active_dir: entities::entry::ActiveModel = directory.into();
					active_dir.aggregate_size = Set(aggregate_size);
					active_dir.child_count = Set(child_count);
					active_dir.file_count = Set(file_count);
					// Bump indexed_at so sync picks up aggregate changes.
					active_dir.indexed_at = Set(Some(chrono::Utc::now()));

					active_dir.update(db).await?;
				}
				Err(e) => {
					tracing::warn!("Failed to aggregate directory {}: {}", directory.name, e);
				}
			}
		}
	}

	Ok(())
}
