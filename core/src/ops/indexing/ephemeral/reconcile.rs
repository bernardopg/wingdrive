//! # Persistent ↔ Ephemeral UUID Reconciliation
//!
//! The ephemeral index assigns random v4 UUIDs at scan time. When the scanned
//! path overlaps an already-indexed (persistent) location, those v4 UUIDs
//! diverge from the stable UUIDs stored in the library database. Tags,
//! selections and metadata attached to the persistent entries then appear
//! missing in ephemeral views, and promoting the path to a managed location
//! orphans the ephemeral identity entirely.
//!
//! This module closes the gap in both directions. The promotion direction
//! (ephemeral → persistent) already exists in `DatabaseStorage`, which reuses
//! `state.get_ephemeral_uuid` when writing entries. What was missing is the
//! browse direction:
//!
//! - [`extract_persistent_uuids_for_path`] loads the persistent UUID for every
//!   entry under a root path in two indexed queries over the `directory_paths`
//!   cache. No recursive parent walks.
//! - [`EphemeralIndex::reconcile_with_persistent`] swaps orphan v4 UUIDs for
//!   their persistent counterparts after a scan completes and reports every
//!   change so callers can emit `ResourceChanged` events.
//! - [`get_or_resolve_uuid`] resolves a single path on demand (search
//!   results, lazy listings) with a database fallback.
//!
//! Overlays are library-scoped (`LibraryId → path → UUID`) because the global
//! ephemeral index is shared across libraries and the same path can carry
//! different identities in each one.

use sea_orm::{ConnectionTrait, DbBackend, DbErr, Statement};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::EphemeralIndex;

/// Outcome counts for a single reconciliation pass.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReconcileStats {
	/// Overlay paths that exist in the ephemeral index (reconciliation candidates)
	pub matched: usize,
	/// Entries whose UUID was swapped for the persistent one
	pub uuid_changed: usize,
	/// Entries under the scanned root with no persistent counterpart (genuinely new)
	pub orphans_detected: usize,
	/// Entries already carrying the persistent UUID
	pub already_consistent: usize,
}

/// Result of [`EphemeralIndex::reconcile_with_persistent`].
#[derive(Debug, Default, Clone)]
pub struct ReconcileResult {
	pub stats: ReconcileStats,
	/// (path, old_uuid, new_uuid) for every swapped entry. `old_uuid` is None
	/// when the entry had no UUID assigned yet (e.g. volume indexing skips
	/// UUID generation) — the frontend still needs an event because lazy
	/// lookups may have handed out a v4 earlier in the session.
	pub changes: Vec<(PathBuf, Option<Uuid>, Uuid)>,
}

/// Load the persistent UUID of every entry under `root` in a single pass.
///
/// Directories resolve through the `directory_paths` cache; files and symlinks
/// resolve by joining their parent's cached path with `name.extension`. The
/// prefix predicate uses an indexed byte range instead of `LIKE` so paths
/// containing `%`, `_` or `\` need no escaping. An empty map means this
/// library has no persistent entries under the scanned root.
pub async fn extract_persistent_uuids_for_path<C: ConnectionTrait>(
	db: &C,
	root: &Path,
) -> Result<HashMap<PathBuf, Uuid>, DbErr> {
	let root_str = root.to_string_lossy().to_string();
	let separator = std::path::MAIN_SEPARATOR;
	let prefix = if root_str.ends_with(separator) {
		root_str.clone()
	} else {
		format!("{root_str}{separator}")
	};
	let upper = format!(
		"{}{}",
		prefix.strip_suffix(separator).unwrap_or(&prefix),
		char::from_u32(separator as u32 + 1).expect("path separator has a successor")
	);
	let values = [root_str.clone().into(), prefix.into(), upper.into()];

	// Directories: directory_paths already caches their absolute path.
	let dir_rows = db
		.query_all(Statement::from_sql_and_values(
			DbBackend::Sqlite,
			r#"SELECT dp.path AS full_path, e.uuid AS uuid
			   FROM directory_paths dp
			   JOIN entries e ON e.id = dp.entry_id
			   WHERE e.uuid IS NOT NULL
			     AND (dp.path = ? OR (dp.path >= ? AND dp.path < ?))"#,
			values.to_vec(),
		))
		.await?;

	let mut uuids = HashMap::new();
	for row in dir_rows {
		let path: String = row.try_get("", "full_path")?;
		let uuid: Uuid = row.try_get("", "uuid")?;
		uuids.insert(PathBuf::from(path), uuid);
	}

	// Files and symlinks: parent directory path + filename.
	let file_rows = db
		.query_all(Statement::from_sql_and_values(
			DbBackend::Sqlite,
			r#"WITH parents AS MATERIALIZED (
			       SELECT entry_id, path FROM directory_paths
			       WHERE path = ? OR (path >= ? AND path < ?)
			   )
			   SELECT dp.path AS parent_path, e.name AS name, e.extension AS extension, e.uuid AS uuid
			   FROM parents dp
			   JOIN entries e ON e.parent_id = dp.entry_id
			   WHERE e.uuid IS NOT NULL AND e.kind != 1"#,
			values.to_vec(),
		))
		.await?;

	for row in file_rows {
		let parent_path: String = row.try_get("", "parent_path")?;
		let name: String = row.try_get("", "name")?;
		let extension: Option<String> = row.try_get("", "extension")?;
		let uuid: Uuid = row.try_get("", "uuid")?;

		let file_name = match extension {
			Some(ext) if !ext.is_empty() => format!("{}.{}", name, ext),
			_ => name,
		};
		uuids.insert(PathBuf::from(parent_path).join(file_name), uuid);
	}

	Ok(uuids)
}

/// Load persistent UUIDs for the immediate children visible in a cached listing.
///
/// This bounded lookup prevents a listing from briefly returning scan-time
/// UUIDs while the background reconciliation traverses a larger volume.
pub async fn lookup_direct_child_uuids<C: ConnectionTrait>(
	db: &C,
	parent: &Path,
	children: &[PathBuf],
) -> Result<HashMap<PathBuf, Uuid>, DbErr> {
	let mut uuids = HashMap::new();
	if children.is_empty() {
		return Ok(uuids);
	}
	let child_set: std::collections::HashSet<&Path> =
		children.iter().map(PathBuf::as_path).collect();

	for chunk in children.chunks(250) {
		let placeholders = std::iter::repeat_n("?", chunk.len())
			.collect::<Vec<_>>()
			.join(",");
		let values = chunk
			.iter()
			.map(|path| path.to_string_lossy().to_string().into())
			.collect::<Vec<sea_orm::Value>>();
		let rows = db.query_all(Statement::from_sql_and_values(
			DbBackend::Sqlite,
			format!("SELECT dp.path AS full_path, e.uuid AS uuid FROM directory_paths dp JOIN entries e ON e.id = dp.entry_id WHERE dp.path IN ({placeholders}) AND e.uuid IS NOT NULL"),
			values,
		)).await?;
		for row in rows {
			let path: String = row.try_get("", "full_path")?;
			let uuid: Uuid = row.try_get("", "uuid")?;
			uuids.insert(PathBuf::from(path), uuid);
		}
	}

	let rows = db
		.query_all(Statement::from_sql_and_values(
			DbBackend::Sqlite,
			r#"SELECT e.name AS name, e.extension AS extension, e.uuid AS uuid
		   FROM directory_paths dp JOIN entries e ON e.parent_id = dp.entry_id
		   WHERE dp.path = ? AND e.kind != 1 AND e.uuid IS NOT NULL"#,
			[parent.to_string_lossy().to_string().into()],
		))
		.await?;
	for row in rows {
		let name: String = row.try_get("", "name")?;
		let extension: Option<String> = row.try_get("", "extension")?;
		let path = parent.join(match extension {
			Some(ext) if !ext.is_empty() => format!("{name}.{ext}"),
			_ => name,
		});
		if child_set.contains(path.as_path()) {
			let uuid: Uuid = row.try_get("", "uuid")?;
			uuids.insert(path, uuid);
		}
	}
	Ok(uuids)
}

/// Lazy single-path fallback used when the overlay has no match.
///
/// The lookup checks a directory's cached path first, then searches the
/// parent's entries by name and extension. Missing individual files must
/// never negatively cache their parent directory.
#[async_trait::async_trait]
pub trait PersistentUuidLookup: Send + Sync {
	async fn lookup(&self, path: &Path) -> Option<Uuid>;
}

/// [`PersistentUuidLookup`] backed by the library database.
pub struct DatabaseUuidLookup {
	conn: sea_orm::DatabaseConnection,
}

impl DatabaseUuidLookup {
	pub fn new(conn: sea_orm::DatabaseConnection) -> Self {
		Self { conn }
	}
}

#[async_trait::async_trait]
impl PersistentUuidLookup for DatabaseUuidLookup {
	async fn lookup(&self, path: &Path) -> Option<Uuid> {
		let path_str = path.to_string_lossy().to_string();

		// Directory entries resolve directly through directory_paths.
		if let Ok(Some(row)) = self
			.conn
			.query_one(Statement::from_sql_and_values(
				DbBackend::Sqlite,
				r#"SELECT e.uuid AS uuid
				   FROM directory_paths dp
				   JOIN entries e ON e.id = dp.entry_id
				   WHERE dp.path = ? AND e.uuid IS NOT NULL"#,
				[path_str.into()],
			))
			.await
		{
			return row.try_get::<Uuid>("", "uuid").ok();
		}

		let parent = path.parent()?;
		let file_name = path.file_name()?.to_string_lossy().to_string();
		let (stem, extension) = match path.extension().and_then(|e| e.to_str()) {
			Some(ext) => (
				path.file_stem()
					.and_then(|s| s.to_str())
					.unwrap_or(&file_name)
					.to_string(),
				Some(ext.to_string()),
			),
			None => (file_name, None),
		};

		// A file matches either with the same extension or, for extensionless
		// names, with a NULL extension column. Building the predicate
		// dynamically avoids binding a typed NULL through sea-orm values.
		let (ext_predicate, ext_value) = match &extension {
			Some(ext) => ("AND e.extension = ?", Some(ext.clone())),
			None => ("AND e.extension IS NULL", None),
		};
		let sql = format!(
			r#"SELECT e.uuid AS uuid
			   FROM entries e
			   JOIN directory_paths dp ON dp.entry_id = e.parent_id
			   WHERE dp.path = ? AND e.name = ? AND e.kind != 1 AND e.uuid IS NOT NULL {ext_predicate}"#
		);

		let mut values: Vec<sea_orm::Value> =
			vec![parent.to_string_lossy().to_string().into(), stem.into()];
		if let Some(ext) = ext_value {
			values.push(ext.into());
		}

		let file_row = self
			.conn
			.query_one(Statement::from_sql_and_values(
				DbBackend::Sqlite,
				sql,
				values,
			))
			.await;

		match file_row {
			Ok(Some(row)) => match row.try_get::<Uuid>("", "uuid") {
				Ok(uuid) => Some(uuid),
				Err(_) => None,
			},
			_ => None,
		}
	}
}

/// Resolve the UUID for `path` with full fallback chain.
///
/// Check the database before returning a scan-time v4 UUID. This repairs
/// early browse results while the background reconciliation is still running.
/// The resolved UUID is cached only for the requesting library.
pub async fn get_or_resolve_uuid(
	index: &mut EphemeralIndex,
	library_id: Uuid,
	path: &Path,
	lookup: &dyn PersistentUuidLookup,
) -> Uuid {
	if let Some(uuid) = index.overlay_uuid(library_id, path) {
		return uuid;
	}
	// A scan-time v4 may already exist. Check persistent storage before
	// returning it, otherwise the fallback cannot repair an early browse.
	if let Some(uuid) = lookup.lookup(path).await {
		index.set_entry_uuid_scoped(library_id, path, uuid);
		return uuid;
	}
	index.get_or_assign_uuid_scoped(library_id, &path.to_path_buf())
}

/// Emit `ResourceChanged` for every UUID that reconciliation swapped.
///
/// The event carries the new UUID as the resource id and the old one in
/// `alternate_ids`, so the frontend replaces its cached record instead of
/// creating a duplicate. `sd_path` is marked no-merge because the identity
/// changed, not the location.
pub fn emit_uuid_reconciled_events(
	event_bus: &crate::infra::event::EventBus,
	index: &EphemeralIndex,
	changes: &[(PathBuf, Option<Uuid>, Uuid)],
	persistent_files: &[crate::domain::file::File],
) {
	use crate::device::get_current_device_slug;
	use crate::domain::addressing::SdPath;
	use crate::domain::file::File;
	use crate::infra::event::{Event, ResourceMetadata};

	if changes.is_empty() {
		return;
	}

	let device_slug = get_current_device_slug();
	let by_id: HashMap<_, _> = persistent_files
		.iter()
		.map(|file| (file.id, file))
		.collect();

	for (path, old_uuid, new_uuid) in changes {
		let Some(metadata) = index.get_entry_ref(path) else {
			continue;
		};
		let content_kind = index.get_content_kind(path);

		let sd_path = SdPath::Physical {
			device_slug: device_slug.clone(),
			path: path.clone(),
		};

		let mut file = by_id
			.get(new_uuid)
			.map(|stored| (*stored).clone())
			.unwrap_or_else(|| File::from_ephemeral(*new_uuid, &metadata, sd_path.clone()));
		file.sd_path = sd_path;
		if file.content_identity.is_none() {
			file.content_kind = content_kind;
		}

		let parent_path = path.parent().map(|p| SdPath::Physical {
			device_slug: device_slug.clone(),
			path: p.to_path_buf(),
		});

		if let Ok(resource_json) = serde_json::to_value(&file) {
			event_bus.emit(Event::ResourceChanged {
				resource_type: "file".to_string(),
				resource: resource_json,
				metadata: Some(ResourceMetadata {
					no_merge_fields: vec!["sd_path".to_string()],
					alternate_ids: old_uuid.iter().copied().collect(),
					affected_paths: parent_path.into_iter().collect(),
				}),
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn sample_metadata(path: PathBuf) -> crate::ops::indexing::database_storage::EntryMetadata {
		crate::ops::indexing::database_storage::EntryMetadata {
			path,
			kind: crate::ops::indexing::state::EntryKind::File,
			size: 10,
			modified: None,
			accessed: None,
			created: None,
			inode: None,
			permissions: None,
			is_hidden: false,
		}
	}

	#[tokio::test]
	async fn database_extraction_and_lazy_file_lookup() {
		use sea_orm::{ConnectionTrait, Database};

		let db = Database::connect("sqlite::memory:").await.unwrap();
		db.execute_unprepared("CREATE TABLE entries (id INTEGER PRIMARY KEY, name TEXT, extension TEXT, kind INTEGER, uuid TEXT, parent_id INTEGER)").await.unwrap();
		db.execute_unprepared(
			"CREATE TABLE directory_paths (entry_id INTEGER PRIMARY KEY, path TEXT)",
		)
		.await
		.unwrap();
		db.execute_unprepared("CREATE INDEX idx_directory_paths_path ON directory_paths(path)")
			.await
			.unwrap();

		let root = PathBuf::from("/tmp/a%_b");
		let child = root.join("nested");
		let root_uuid = Uuid::new_v4();
		let child_uuid = Uuid::new_v4();
		let file_uuid = Uuid::new_v4();
		let outside_uuid = Uuid::new_v4();
		for (id, name, ext, kind, uuid, parent) in [
			(1, "a%_b", None, 1, root_uuid, None),
			(2, "nested", None, 1, child_uuid, Some(1)),
			(3, "photo", Some("jpg"), 0, file_uuid, Some(2)),
			(4, "outside", Some("jpg"), 0, outside_uuid, Some(5)),
			(5, "aZZb", None, 1, Uuid::new_v4(), None),
		] {
			let parent = parent.map(|p: i32| p.to_string()).unwrap_or("NULL".into());
			let ext = ext.map(|e| format!("'{e}'")).unwrap_or("NULL".into());
			let hex = uuid
				.as_bytes()
				.iter()
				.map(|b| format!("{b:02x}"))
				.collect::<String>();
			db.execute_unprepared(&format!(
				"INSERT INTO entries VALUES ({id}, '{name}', {ext}, {kind}, X'{hex}', {parent})"
			))
			.await
			.unwrap();
		}
		for (id, path) in [
			(1, root.clone()),
			(2, child.clone()),
			(5, PathBuf::from("/tmp/aZZb")),
		] {
			db.execute_unprepared(&format!(
				"INSERT INTO directory_paths VALUES ({id}, '{}')",
				path.display()
			))
			.await
			.unwrap();
		}

		let uuids = extract_persistent_uuids_for_path(&db, &root).await.unwrap();
		assert_eq!(uuids.get(&root), Some(&root_uuid));
		assert_eq!(uuids.get(&child), Some(&child_uuid));
		let file = child.join("photo.jpg");
		assert_eq!(uuids.get(&file), Some(&file_uuid));
		assert_eq!(uuids.len(), 3);

		let root_children =
			lookup_direct_child_uuids(&db, &root, &[child.clone(), child.join("photo.jpg")])
				.await
				.unwrap();
		assert_eq!(root_children.len(), 1);
		assert_eq!(root_children.get(&child), Some(&child_uuid));
		let nested_children = lookup_direct_child_uuids(&db, &child, std::slice::from_ref(&file))
			.await
			.unwrap();
		assert_eq!(nested_children.get(&file), Some(&file_uuid));

		let lookup = DatabaseUuidLookup::new(db.clone());
		assert_eq!(lookup.lookup(&file).await, Some(file_uuid));
		assert_eq!(lookup.lookup(&child).await, Some(child_uuid));
		assert_eq!(lookup.lookup(&child.join("missing.jpg")).await, None);
		assert_eq!(lookup.lookup(&file).await, Some(file_uuid));

		let mut index = EphemeralIndex::default();
		index
			.add_entry(file.clone(), Uuid::new_v4(), sample_metadata(file.clone()))
			.unwrap();
		let library_id = Uuid::new_v4();
		assert_eq!(
			get_or_resolve_uuid(&mut index, library_id, &file, &lookup).await,
			file_uuid
		);
		assert_eq!(
			index.get_entry_uuid_scoped(library_id, &file),
			Some(file_uuid)
		);
	}

	#[tokio::test]
	#[ignore = "run explicitly for 100K-entry reconciliation timing"]
	async fn reconciliation_of_100k_entries_is_bounded() {
		use sea_orm::{ConnectionTrait, Database};

		let mut index = EphemeralIndex::default();
		let library_id = Uuid::new_v4();
		let root = PathBuf::from("/benchmark");
		let entries = (0..100_000)
			.map(|i| {
				let path = root.join(format!("file_{i}.txt"));
				(path.clone(), Some(Uuid::new_v4()), sample_metadata(path))
			})
			.collect();
		index.add_entries_batch(entries).unwrap();

		let db = Database::connect("sqlite::memory:").await.unwrap();
		db.execute_unprepared("CREATE TABLE entries (id INTEGER PRIMARY KEY, name TEXT, extension TEXT, kind INTEGER, uuid BLOB, parent_id INTEGER)").await.unwrap();
		db.execute_unprepared(
			"CREATE TABLE directory_paths (entry_id INTEGER PRIMARY KEY, path TEXT)",
		)
		.await
		.unwrap();
		db.execute_unprepared("CREATE INDEX idx_directory_paths_path ON directory_paths(path)")
			.await
			.unwrap();
		db.execute_unprepared("CREATE INDEX idx_entries_parent_id ON entries(parent_id)")
			.await
			.unwrap();
		db.execute_unprepared(
			"INSERT INTO entries VALUES (1, 'benchmark', NULL, 1, randomblob(16), NULL)",
		)
		.await
		.unwrap();
		db.execute_unprepared("INSERT INTO directory_paths VALUES (1, '/benchmark')")
			.await
			.unwrap();
		db.execute_unprepared("WITH RECURSIVE seq(n) AS (SELECT 0 UNION ALL SELECT n+1 FROM seq WHERE n<99999) INSERT INTO entries SELECT n+2, 'file_'||n, 'txt', 0, randomblob(16), 1 FROM seq").await.unwrap();

		let start = std::time::Instant::now();
		let mut overlay = extract_persistent_uuids_for_path(&db, &root).await.unwrap();
		assert_eq!(overlay.len(), 100_001);
		overlay.remove(&root);
		let result = index.reconcile_with_persistent(library_id, &root, &overlay);
		let elapsed = start.elapsed();
		eprintln!("100K SQL extraction + UUID reconciliation: {elapsed:?}");
		assert_eq!(result.stats.matched, 100_000);
		assert_eq!(result.stats.uuid_changed, 100_000);
		assert!(
			elapsed < std::time::Duration::from_secs(2),
			"100K reconciliation took {elapsed:?}"
		);
	}

	#[test]
	fn reconcile_swaps_orphan_v4_for_persistent_uuid() {
		let mut index = EphemeralIndex::default();
		let library_id = Uuid::new_v4();
		let root = PathBuf::from("/photos");
		let file_path = root.join("cat.jpg");
		let persistent_uuid = Uuid::new_v4();

		index
			.add_entry(
				file_path.clone(),
				Uuid::new_v4(),
				sample_metadata(file_path.clone()),
			)
			.unwrap();
		let orphan_uuid = index.get_entry_uuid(&file_path).unwrap();
		assert_ne!(orphan_uuid, persistent_uuid);

		let overlay: HashMap<PathBuf, Uuid> =
			[(file_path.clone(), persistent_uuid)].into_iter().collect();
		let result = index.reconcile_with_persistent(library_id, &root, &overlay);

		assert_eq!(result.stats.uuid_changed, 1);
		assert_eq!(result.stats.already_consistent, 0);
		assert_eq!(result.stats.matched, 1);
		assert_eq!(result.changes.len(), 1);
		assert_eq!(result.changes[0].1, Some(orphan_uuid));
		assert_eq!(result.changes[0].2, persistent_uuid);
		assert_eq!(
			index.get_entry_uuid_scoped(library_id, &file_path),
			Some(persistent_uuid)
		);
		assert_eq!(index.get_entry_uuid(&file_path), Some(orphan_uuid));
	}

	#[test]
	fn reconcile_is_idempotent() {
		let mut index = EphemeralIndex::default();
		let library_id = Uuid::new_v4();
		let root = PathBuf::from("/photos");
		let file_path = root.join("cat.jpg");
		let persistent_uuid = Uuid::new_v4();

		index
			.add_entry(
				file_path.clone(),
				persistent_uuid,
				sample_metadata(file_path.clone()),
			)
			.unwrap();

		let overlay: HashMap<PathBuf, Uuid> =
			[(file_path.clone(), persistent_uuid)].into_iter().collect();
		let result = index.reconcile_with_persistent(library_id, &root, &overlay);

		assert_eq!(result.stats.already_consistent, 1);
		assert_eq!(result.stats.uuid_changed, 0);
		assert!(result.changes.is_empty());
	}

	#[test]
	fn reconcile_counts_orphans_and_unmatched_overlay() {
		let mut index = EphemeralIndex::default();
		let library_id = Uuid::new_v4();
		let root = PathBuf::from("/photos");

		// New file on disk, never indexed persistently.
		let orphan_path = root.join("new-dog.jpg");
		index
			.add_entry(
				orphan_path.clone(),
				Uuid::new_v4(),
				sample_metadata(orphan_path.clone()),
			)
			.unwrap();

		// Persistently indexed file that no longer exists on disk.
		let deleted_path = root.join("deleted.jpg");
		let overlay: HashMap<PathBuf, Uuid> =
			[(deleted_path, Uuid::new_v4())].into_iter().collect();

		let result = index.reconcile_with_persistent(library_id, &root, &overlay);

		assert_eq!(result.stats.matched, 0);
		assert_eq!(result.stats.orphans_detected, 1);
		// Orphan keeps its v4 UUID.
		assert!(index.get_entry_uuid(&orphan_path).is_some());
	}

	#[test]
	fn scoped_uuid_resolution_prefers_overlay_then_entry() {
		let mut index = EphemeralIndex::default();
		let library_id = Uuid::new_v4();
		let root = PathBuf::from("/photos");
		let file_path = root.join("cat.jpg");
		let persistent_uuid = Uuid::new_v4();

		index
			.add_entry(
				file_path.clone(),
				Uuid::new_v4(),
				sample_metadata(file_path.clone()),
			)
			.unwrap();

		let overlay: HashMap<PathBuf, Uuid> =
			[(file_path.clone(), persistent_uuid)].into_iter().collect();
		index.set_uuid_overlay(library_id, overlay);

		// Entry UUID was already reconciled in place.
		assert_eq!(
			index.get_or_assign_uuid_scoped(library_id, &file_path),
			persistent_uuid
		);
		assert_eq!(index.overlay_uuid_count(library_id), 1);

		// Unknown path falls back to v4 generation.
		let unknown = root.join("ghost.png");
		let assigned = index.get_or_assign_uuid_scoped(library_id, &unknown);
		assert_ne!(assigned, persistent_uuid);
	}

	#[test]
	fn stale_persistent_uuid_is_removed_when_root_becomes_unmanaged() {
		let mut index = EphemeralIndex::default();
		let path = PathBuf::from("/shared/removed.txt");
		let root = PathBuf::from("/shared");
		index
			.add_entry(path.clone(), Uuid::new_v4(), sample_metadata(path.clone()))
			.unwrap();
		let library = Uuid::new_v4();
		let persistent = Uuid::new_v4();
		index.reconcile_with_persistent(library, &root, &[(path.clone(), persistent)].into());
		let result = index.reconcile_with_persistent(library, &root, &HashMap::new());
		assert_eq!(result.stats.uuid_changed, 1);
		assert_eq!(result.changes[0].1, Some(persistent));
		assert_ne!(
			index.get_entry_uuid_scoped(library, &path),
			Some(persistent)
		);
		assert_eq!(index.overlay_uuid(library, &path), None);
		assert_eq!(
			index
				.reconcile_with_persistent(library, &root, &HashMap::new())
				.stats
				.uuid_changed,
			0
		);
	}

	#[test]
	fn overlapping_libraries_keep_independent_identities() {
		let mut index = EphemeralIndex::default();
		let path = PathBuf::from("/shared/image.jpg");
		let root = PathBuf::from("/shared");
		index
			.add_entry(path.clone(), Uuid::new_v4(), sample_metadata(path.clone()))
			.unwrap();
		let a = Uuid::new_v4();
		let b = Uuid::new_v4();
		let a_persistent = Uuid::new_v4();
		let b_persistent = Uuid::new_v4();
		index.reconcile_with_persistent(a, &root, &[(path.clone(), a_persistent)].into());
		index.reconcile_with_persistent(b, &root, &[(path.clone(), b_persistent)].into());
		assert_eq!(index.get_or_assign_uuid_scoped(a, &path), a_persistent);
		assert_eq!(index.get_or_assign_uuid_scoped(b, &path), b_persistent);
		assert_ne!(a_persistent, b_persistent);
	}

	#[test]
	fn overlay_for_one_root_preserves_another_root() {
		let mut index = EphemeralIndex::default();
		let library_id = Uuid::new_v4();
		let a = PathBuf::from("/old/path.jpg");
		let b = PathBuf::from("/new/path.jpg");
		for path in [&a, &b] {
			index
				.add_entry(path.clone(), Uuid::new_v4(), sample_metadata(path.clone()))
				.unwrap();
		}
		let first = Uuid::new_v4();
		index.set_uuid_overlay(library_id, [(a.clone(), first)].into());
		index.set_uuid_overlay(library_id, [(b, Uuid::new_v4())].into());
		assert_eq!(index.overlay_uuid_count(library_id), 2);
		assert_eq!(index.get_entry_uuid_scoped(library_id, &a), Some(first));
	}
}
