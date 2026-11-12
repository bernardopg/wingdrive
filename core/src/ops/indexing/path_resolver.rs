//! # Path Resolution via directory_paths Cache
//!
//! Resolves full filesystem paths for entries without walking parent_id chains. The directory_paths
//! table caches absolute paths for all directories, making lookups O(1) instead of O(depth). Files
//! are resolved by joining their parent's cached path with the filename. This table is updated during
//! indexing and move operations to keep paths in sync with the entry hierarchy.

use std::path::{Path, PathBuf};

use sea_orm::{prelude::*, Condition, ConnectionTrait, QuerySelect, Statement};

use crate::{
	domain::addressing::WingPath,
	infra::db::entities::{device, directory_paths, entry, location, DirectoryPaths, Entry},
};

pub struct PathResolver;

impl PathResolver {
	/// Resolves the absolute path by looking up directories in the cache or reconstructing file paths.
	pub async fn get_full_path<C: ConnectionTrait>(
		db: &C,
		entry_id: i32,
	) -> Result<PathBuf, DbErr> {
		let entry = Entry::find_by_id(entry_id)
			.one(db)
			.await?
			.ok_or_else(|| DbErr::RecordNotFound(format!("Entry {} not found", entry_id)))?;

		match entry.entry_kind() {
			crate::infra::db::entities::entry::EntryKind::Directory => {
				let dir_path = DirectoryPaths::find_by_id(entry_id)
					.one(db)
					.await?
					.ok_or_else(|| {
						DbErr::RecordNotFound(format!(
							"Directory path not found for entry {}",
							entry_id
						))
					})?;
				Ok(PathBuf::from(dir_path.path))
			}
			_ => {
				if let Some(parent_id) = entry.parent_id {
					let parent_path = DirectoryPaths::find_by_id(parent_id)
						.one(db)
						.await?
						.ok_or_else(|| {
							DbErr::RecordNotFound(format!(
								"Parent directory path not found for entry {}",
								parent_id
							))
						})?;

					let full_filename = if let Some(ext) = &entry.extension {
						format!("{}.{}", entry.name, ext)
					} else {
						entry.name.clone()
					};

					Ok(PathBuf::from(parent_path.path).join(full_filename))
				} else {
					let full_filename = if let Some(ext) = &entry.extension {
						format!("{}.{}", entry.name, ext)
					} else {
						entry.name.clone()
					};
					Ok(PathBuf::from(full_filename))
				}
			}
		}
	}

	/// Fetches the cached path string directly from directory_paths without entry lookup.
	pub async fn get_directory_path<C: ConnectionTrait>(
		db: &C,
		directory_id: i32,
	) -> Result<String, DbErr> {
		DirectoryPaths::find_by_id(directory_id)
			.one(db)
			.await?
			.map(|dp| dp.path)
			.ok_or_else(|| {
				DbErr::RecordNotFound(format!(
					"Directory path not found for entry {}",
					directory_id
				))
			})
	}

	/// Constructs the path string for a new directory by joining its parent's path with its name.
	///
	/// Used during indexing to populate the directory_paths table for newly discovered directories.
	pub async fn build_directory_path<C: ConnectionTrait>(
		db: &C,
		parent_id: Option<i32>,
		name: &str,
	) -> Result<String, DbErr> {
		if let Some(parent_id) = parent_id {
			let parent_path = Self::get_directory_path(db, parent_id).await?;
			Ok(crate::domain::addressing::join_entry_path(
				&parent_path,
				name,
			))
		} else {
			Ok(name.to_string())
		}
	}

	/// Resolves paths for multiple entries in batched queries to minimize database round-trips.
	pub async fn get_paths_batch<C: ConnectionTrait>(
		db: &C,
		entry_ids: Vec<i32>,
	) -> Result<Vec<(i32, PathBuf)>, DbErr> {
		let mut entries: Vec<entry::Model> = Vec::new();
		let chunk_size: usize = 900;
		for chunk in entry_ids.chunks(chunk_size) {
			let mut batch = Entry::find()
				.filter(entry::Column::Id.is_in(chunk.to_vec()))
				.all(db)
				.await?;
			entries.append(&mut batch);
		}

		let mut results = Vec::with_capacity(entries.len());

		let mut directory_ids = Vec::new();
		let mut file_entries = Vec::new();

		for entry in entries {
			match entry.entry_kind() {
				crate::infra::db::entities::entry::EntryKind::Directory => {
					directory_ids.push(entry.id);
				}
				_ => {
					file_entries.push(entry);
				}
			}
		}

		if !directory_ids.is_empty() {
			let mut dir_paths: Vec<directory_paths::Model> = Vec::new();
			for chunk in directory_ids.chunks(chunk_size) {
				let mut batch = DirectoryPaths::find()
					.filter(directory_paths::Column::EntryId.is_in(chunk.to_vec()))
					.all(db)
					.await?;
				dir_paths.append(&mut batch);
			}

			for dir_path in dir_paths {
				results.push((dir_path.entry_id, PathBuf::from(dir_path.path)));
			}
		}

		if !file_entries.is_empty() {
			let parent_ids: Vec<i32> = file_entries.iter().filter_map(|e| e.parent_id).collect();

			let mut parent_paths: Vec<directory_paths::Model> = Vec::new();
			for chunk in parent_ids.chunks(chunk_size) {
				let mut batch = DirectoryPaths::find()
					.filter(directory_paths::Column::EntryId.is_in(chunk.to_vec()))
					.all(db)
					.await?;
				parent_paths.append(&mut batch);
			}

			let parent_map: std::collections::HashMap<i32, String> = parent_paths
				.into_iter()
				.map(|dp| (dp.entry_id, dp.path))
				.collect();

			for file_entry in file_entries {
				let path = if let Some(parent_id) = file_entry.parent_id {
					if let Some(parent_path) = parent_map.get(&parent_id) {
						PathBuf::from(parent_path).join(&file_entry.name)
					} else {
						PathBuf::from(&file_entry.name)
					}
				} else {
					PathBuf::from(&file_entry.name)
				};
				results.push((file_entry.id, path));
			}
		}

		Ok(results)
	}

	/// Bulk-updates descendant directory paths after moving a directory tree.
	///
	/// Uses a single SQL REPLACE to rewrite all paths under the moved directory's old prefix.
	/// Should be called after updating the moved directory's entry.parent_id and directory_paths.path.
	pub async fn update_descendant_paths<C: ConnectionTrait>(
		db: &C,
		moved_directory_id: i32,
		old_path: &str,
		new_path: &str,
	) -> Result<u64, DbErr> {
		let sql = r#"
            UPDATE directory_paths
            SET path = REPLACE(path, ?, ?)
            WHERE path LIKE ? || '/%'
        "#;

		let result = db
			.execute_raw(Statement::from_sql_and_values(
				db.get_database_backend(),
				sql,
				vec![old_path.into(), new_path.into(), old_path.into()],
			))
			.await?;

		Ok(result.rows_affected())
	}

	/// Resolve an WingPath to its entry in the database (reverse lookup)
	/// Returns the entry with full metadata (size, file_count, aggregate_size, etc.)
	pub async fn resolve_to_entry<C: ConnectionTrait>(
		db: &C,
		path: &WingPath,
	) -> Result<Option<entry::Model>, DbErr> {
		match path {
			WingPath::Physical {
				device_slug,
				path: local_path,
			} => {
				// The `local` placeholder and the device UUID name this device too
				let device = if path.is_local() {
					device::Entity::find()
						.filter(device::Column::Uuid.eq(crate::device::get_current_device_id()))
						.one(db)
						.await?
				} else {
					device::Entity::find()
						.filter(device::Column::Slug.eq(device_slug.as_str()))
						.one(db)
						.await?
				};
				match device {
					Some(device) => Self::resolve_physical_to_entry(db, &device, local_path).await,
					None => Ok(None),
				}
			}
			WingPath::Cloud { .. } => {
				// TODO: Implement cloud path resolution
				Ok(None)
			}
			WingPath::Content { content_id } => {
				// Query by content_id
				entry::Entity::find()
					.filter(entry::Column::ContentId.eq(*content_id))
					.one(db)
					.await
			}
			WingPath::Sidecar { .. } => {
				// Sidecars don't have entries
				Ok(None)
			}
		}
	}

	/// Resolve a Physical path on `device` to its entry
	async fn resolve_physical_to_entry<C: ConnectionTrait>(
		db: &C,
		device: &device::Model,
		path: &PathBuf,
	) -> Result<Option<entry::Model>, DbErr> {
		// Every location of the device, including those whose volume is not
		// resolved yet (`volume_id` is NULL until the first index)
		let locations = location::Entity::find()
			.filter(location::Column::DeviceId.eq(device.id))
			.all(db)
			.await?;

		for location in locations {
			let Some(entry_id) = location.entry_id else {
				continue;
			};

			// Get location root path from directory_paths
			let location_path = Self::get_full_path(db, entry_id).await?;

			// Check if target path is within this location. Comparing components
			// keeps a location `/loc` from claiming paths under `/loc2`.
			if path.starts_with(&location_path) {
				// Exact match - return location root
				if path == &location_path {
					return entry::Entity::find_by_id(entry_id).one(db).await;
				}

				// Find entry by traversing path
				return Self::find_entry_by_path(db, entry_id, path, &location_path).await;
			}
		}

		Ok(None)
	}

	/// Find an entry by traversing from a parent directory
	async fn find_entry_by_path<C: ConnectionTrait>(
		db: &C,
		parent_entry_id: i32,
		target_path: &Path,
		parent_path: &Path,
	) -> Result<Option<entry::Model>, DbErr> {
		// Get relative path
		let relative_path = match target_path.strip_prefix(parent_path) {
			Ok(rel) => rel,
			Err(_) => return Ok(None),
		};

		let components: Vec<&str> = relative_path
			.components()
			.filter_map(|c| c.as_os_str().to_str())
			.collect();

		if components.is_empty() {
			return Ok(None);
		}

		// Traverse hierarchy
		let mut current_parent_id = Some(parent_entry_id);
		let component_count = components.len();

		for (index, component) in components.iter().enumerate() {
			let Some(parent_id) = current_parent_id else {
				return Ok(None);
			};

			// Directories keep their full name; files store the stem and the
			// lowercase extension. Either form can match, so a folder `v1.2` is
			// found and a folder `notes` is not confused with `notes.txt`.
			let mut child = None;
			for spaces in [None, Some('\u{202f}'), Some('\u{00a0}')] {
				// The database may hold names with Unicode spaces
				let candidate = match spaces {
					Some(space) if component.contains(' ') => {
						component.replace(' ', space.encode_utf8(&mut [0; 4]))
					}
					Some(_) => continue,
					None => String::from(*component),
				};
				child = entry::Entity::find()
					.filter(entry::Column::ParentId.eq(parent_id))
					.filter(component_condition(&candidate))
					.one(db)
					.await?;
				if child.is_some() {
					break;
				}
			}

			match child {
				Some(c) => {
					current_parent_id = Some(c.id);
					// If this is the last component, return the entry
					if index == component_count - 1 {
						return Ok(Some(c));
					}
				}
				None => return Ok(None),
			}
		}

		Ok(None)
	}
}

/// Matches an entry named `component`: a directory or extensionless entry with
/// that full name, or a file whose stem and extension make it up.
fn component_condition(component: &str) -> Condition {
	let whole = Condition::all()
		.add(entry::Column::Name.eq(component))
		.add(entry::Column::Extension.is_null());
	let path = Path::new(component);
	match (
		path.file_stem().and_then(|stem| stem.to_str()),
		path.extension().and_then(|ext| ext.to_str()),
	) {
		(Some(stem), Some(extension)) => Condition::any().add(whole).add(
			Condition::all()
				.add(entry::Column::Name.eq(stem))
				.add(entry::extension_lookup(extension)),
		),
		_ => whole,
	}
}
